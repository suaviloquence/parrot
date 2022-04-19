use std::{
	net::IpAddr,
	path::PathBuf,
	process::{self, Child, Command},
};

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
	String(String),
	IP,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Action {
	exec: String,
	args: Vec<Token>,
}

impl Action {
	fn command(&self, ip: IpAddr) -> Command {
		let mut command = Command::new(&self.exec);
		command.args(self.args.iter().map(|x| match x {
			Token::String(s) => s.clone(),
			Token::IP => ip.to_string(),
		}));
		command
	}
	pub fn run(&self, ip: IpAddr) -> std::io::Result<Child> {
		self.command(ip).spawn()
	}
}

impl TryFrom<String> for Action {
	type Error = &'static str;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		let mut split = value.split_whitespace();

		let exec = split.next().ok_or("Empty action field.")?.to_string();

		let args = split
			.map(|arg| match arg {
				"%IP" => Token::IP,
				arg => Token::String(arg.to_string()),
			})
			.collect();

		Ok(Self { exec, args })
	}
}

#[derive(Debug, PartialEq, Clone)]
pub enum PeerHost {
	IP(IpAddr),
	HOST,
	INFER,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Config {
	pub notify: Action,
	pub host: String,
	pub server_port: u16,
	pub peer_port: u16,
	pub info_hash: [u8; 20],
	pub file: Option<PathBuf>,
	pub expected_ip: IpAddr,
	pub peer_host: PeerHost,
}

fn next_arg(args: &mut impl Iterator<Item = String>) -> Result<String, &'static str> {
	match args.next() {
		Some(s) => Ok(s),
		None => Err("Missing expected argument."),
	}
}

impl Config {
	pub fn load(mut args: impl Iterator<Item = String>) -> Result<Self, &'static str> {
		let mut command = Err("Missing command.");
		let mut info_hash = Err("Missing info hash.");
		let mut expected_ip = Err("Missing expected ip.");

		// DEFAULTS
		let mut host = "127.0.0.1".to_string();
		let mut server_port = 3000;
		let mut peer_port = 16384;
		let mut file = None;
		let mut peer_host = PeerHost::INFER;

		loop {
			match args.next().as_deref() {
				Some("-n" | "--notify") => match args.next() {
					Some(c) => command = Action::try_from(c),
					None => return Err("Missing value for \"notify\""),
				},
				Some("-i" | "--info") => {
					let arg = next_arg(&mut args)?;
					let mut chars = arg.chars();
					let mut info_vec: Vec<u8> = Vec::new();
					while let (Some(a), Some(b)) = (chars.next(), chars.next()) {
						match (a.to_digit(16), b.to_digit(16)) {
							(Some(a), Some(b)) => {
								// max of a and b is both 15, so the max of this expression is (15 * 16) + 15 = 255 < 2^8
								info_vec.push(((a * 16) + b).try_into().unwrap())
							}
							_ => return Err("Invalid info hash."),
						}
					}
					info_hash = info_vec
						.try_into()
						.map_err(|_| "Invalid length of info hash.");
				}
				Some("-h" | "--host") => host = next_arg(&mut args)?,
				Some("-s" | "--server-port") => {
					server_port = next_arg(&mut args)?
						.parse()
						.map_err(|_| "Invalid server port (must be a number 0 < port < 65536)")?
				}
				Some("-p" | "--peer-port") => {
					peer_port = next_arg(&mut args)?
						.parse()
						.map_err(|_| "Invalid peer port (must be a number 0 < port < 65536)")?
				}
				Some("-f" | "--file") => match args.next() {
					Some(f) => {
						let path = PathBuf::from(f);
						if path.is_file() {
							file = Some(path);
							info_hash = Ok([0; 20]); // placeholder: if file is set, info_hash will always be overwritten
							 // TODO find a more elegant solution
						} else {
							return Err("Argument is not a file.");
						}
					}
					None => return Err("Missing value for \"file\""),
				},
				Some("-e" | "--expected-ip") => {
					expected_ip = next_arg(&mut args)?
						.parse()
						.map_err(|_| "Invalid IP address.")
				}
				Some("--peer-host") => {
					peer_host = match next_arg(&mut args).as_deref() {
						Ok("infer") => Ok(PeerHost::INFER),
						Ok("host") => Ok(PeerHost::HOST),
						Ok(ip) => ip
							.parse()
							.map(|ip| PeerHost::IP(ip))
							.map_err(|_| "Invalid IP address"),
						_ => Err("Invalid peer host."),
					}?
				}
				Some(_) => return Err("Unexpected token."),
				None => break,
			}
		}

		Ok(Self {
			notify: command?,
			info_hash: info_hash?,
			host,
			peer_host,
			server_port,
			peer_port,
			file,
			expected_ip: expected_ip?,
		})
	}

	pub fn load_or_exit() -> Self {
		let mut args = std::env::args();
		let filename = args.next().unwrap();

		match Self::load(args) {
			Ok(c) => c,
			Err(e) => {
				println!(
					r#"ERROR: {:?}
					
run {} --help to print a help menu.
				"#,
					e, filename,
				);
				process::exit(1)
			}
		}
	}
}

#[cfg(test)]
impl Default for Config {
	fn default() -> Self {
		Self {
			notify: Action {
				exec: String::from(""),
				args: vec![],
			},
			host: "127.0.0.1".into(),
			peer_host: PeerHost::INFER,
			server_port: 3000,
			peer_port: 16384,
			info_hash: [1; 20],
			file: None,
			expected_ip: "127.0.0.1".parse().unwrap(),
		}
	}
}

#[cfg(test)]
mod tests {
	use std::net::IpAddr;

	use crate::config::{Config, PeerHost};

	use super::{Action, Token};

	macro_rules! args {
		($($arg: expr$(, )?)*) => {{
			#[allow(unused_mut)]
			let mut vec = Vec::new();

			$(
				vec.push($arg.to_string());
			)*

			vec.into_iter()
		}};
	}

	#[test]
	fn test_action_from() {
		assert_eq!(
			Action::try_from("ls -la".to_string()),
			Ok(Action {
				exec: "ls".into(),
				args: vec![Token::String("-la".into())]
			})
		);

		assert_eq!(Action::try_from(String::new()), Err("Empty action field."));

		assert_eq!(
			Action::try_from("abc de %IP".to_string()),
			Ok(Action {
				exec: "abc".to_string(),
				args: vec![Token::String("de".into()), Token::IP]
			})
		);
	}

	#[test]
	fn test_action_command() {
		let one = Action::try_from("ls -la".to_string())
			.unwrap()
			.command(IpAddr::V4("127.0.0.1".parse().unwrap()));
		assert_eq!(one.get_program(), "ls");
		assert_eq!(one.get_args().into_iter().collect::<Vec<_>>(), vec!["-la"]);

		let two = Action::try_from("echo Your IP is %IP".to_string())
			.unwrap()
			.command(IpAddr::V6("::1".parse().unwrap()));

		assert_eq!(two.get_program(), "echo");
		assert_eq!(
			two.get_args().into_iter().collect::<Vec<_>>(),
			vec!["Your", "IP", "is", "::1"]
		)
	}

	#[test]
	fn test_config_from() {
		assert_eq!(
			Config::load(
				vec![
					"-n",
					"ls -la",
					"--info",
					"ffffffffffffffffffffffffffffffffffffffff",
					"--expected-ip",
					"127.0.0.1"
				]
				.into_iter()
				.map(&str::to_string)
			),
			Ok(Config {
				info_hash: [0xff; 20],
				notify: Action {
					exec: "ls".into(),
					args: vec![Token::String("-la".into())],
				},
				host: "127.0.0.1".into(),
				peer_host: PeerHost::INFER,
				server_port: 3000,
				peer_port: 16384,
				file: None,
				expected_ip: "127.0.0.1".parse().unwrap(),
			})
		);

		assert_eq!(Config::load([].into_iter()), Err("Missing command."));

		assert_eq!(
			Config::load(["-n"].into_iter().map(&str::to_string)),
			Err("Missing value for \"notify\"")
		);

		assert_eq!(
			Config::load(["-n", ""].into_iter().map(&str::to_string)),
			Err("Empty action field.")
		);

		assert_eq!(
			Config::load(["-n", "ls -la"].into_iter().map(&str::to_string)),
			Err("Missing info hash.")
		);

		assert_eq!(
			Config::load(["-n", "ls -la", "-i"].into_iter().map(&str::to_string)),
			Err("Missing expected argument.")
		);

		assert_eq!(
			Config::load(
				["-n", "ls -la", "-i", "abc"]
					.into_iter()
					.map(&str::to_string)
			),
			Err("Invalid length of info hash.")
		);

		assert_eq!(
			Config::load(
				["-n", "ls -la", "-i", "####"]
					.into_iter()
					.map(&str::to_string)
			),
			Err("Invalid info hash.")
		);

		assert_eq!(
			Config::load(
				["-n", "ls -la", "-i", "00000000000000000000000000000000000000000000000000000000000000000000000000000000", "-f", "this file doesn't exist"]
					.into_iter()
					.map(&str::to_string)
			),
			Err("Argument is not a file.")
		);

		// directory
		assert_eq!(
			Config::load(
				["-n", "ls -la", "-i", "00000000000000000000000000000000000000000000000000000000000000000000000000000000", "-f", "src"]
					.into_iter()
					.map(&str::to_string)
			),
			Err("Argument is not a file.")
		);

		assert_eq!(
			Config::load(args!(
				"-n",
				"true",
				"-i",
				"0000000000000000000000000000000000000000",
				"-e",
				"127.3"
			)),
			Err("Invalid IP address.")
		)
	}
}
