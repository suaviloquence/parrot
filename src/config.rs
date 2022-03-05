use std::{
	net::IpAddr,
	path::PathBuf,
	process::{self, Child, Command},
};

#[derive(Debug, PartialEq)]
pub enum Error {
	EmptyAction,
	MissingCommand,
	MissingInfoHash,
	MissingArgument,
	InvalidArgument,
	UnexpectedArgument,
	InvalidFile,
}

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
	type Error = Error;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		let mut split = value.split_whitespace();

		let exec = split.next().ok_or(Error::EmptyAction)?.to_string();

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
pub struct Config {
	pub notify: Action,
	pub host: String,
	pub server_port: u16,
	pub peer_port: u16,
	pub info_hash: [u8; 20],
	pub file: Option<PathBuf>,
}

fn next_arg(args: &mut impl Iterator<Item = String>) -> Result<String, Error> {
	match args.next() {
		Some(s) => Ok(s),
		None => Err(Error::MissingArgument),
	}
}

impl Config {
	pub fn load(mut args: impl Iterator<Item = String>) -> Result<Self, Error> {
		let mut command = Err(Error::MissingCommand);
		let mut info_hash = Err(Error::MissingInfoHash);

		// DEFAULTS
		let mut host = "127.0.0.1".to_string();
		let mut server_port = 3000;
		let mut peer_port = 16384;
		let mut file = None;

		loop {
			match args.next().as_deref() {
				Some("-n" | "--notify") => match args.next() {
					Some(c) => command = Action::try_from(c),
					None => return Err(Error::MissingArgument),
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
							_ => return Err(Error::InvalidArgument),
						}
					}
					info_hash = info_vec.try_into().map_err(|_| Error::InvalidArgument);
				}
				Some("-h" | "--host") => host = next_arg(&mut args)?,
				Some("-s" | "--server-port") => {
					server_port = next_arg(&mut args)?
						.parse()
						.map_err(|_| Error::InvalidArgument)?
				}
				Some("-p" | "--peer-port") => {
					peer_port = next_arg(&mut args)?
						.parse()
						.map_err(|_| Error::InvalidArgument)?
				}
				Some("-f" | "--file") => match args.next() {
					Some(f) => {
						let path = PathBuf::from(f);
						if path.is_file() {
							file = Some(path);
							info_hash = Ok([0; 20]); // placeholder: if file is set, info_hash will always be overwritten
							 // TODO find a more elegant solution
						} else {
							return Err(Error::InvalidFile);
						}
					}
					None => return Err(Error::MissingArgument),
				},
				Some(_) => return Err(Error::UnexpectedArgument),
				None => break,
			}
		}

		Ok(Self {
			notify: command?,
			info_hash: info_hash?,
			host,
			server_port,
			peer_port,
			file,
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
pub fn test_config() -> Config {
	Config {
		notify: Action {
			exec: String::from(""),
			args: vec![],
		},
		host: "127.0.0.1".into(),
		server_port: 3000,
		peer_port: 16384,
		info_hash: [1; 20],
		file: None,
	}
}

#[cfg(test)]
mod tests {
	use std::net::IpAddr;

	use crate::config::Config;

	use super::{Action, Error, Token};

	#[test]
	fn test_action_from() {
		assert_eq!(
			Action::try_from("ls -la".to_string()),
			Ok(Action {
				exec: "ls".into(),
				args: vec![Token::String("-la".into())]
			})
		);

		assert_eq!(Action::try_from(String::new()), Err(Error::EmptyAction));

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
					"ffffffffffffffffffffffffffffffffffffffff"
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
				server_port: 3000,
				peer_port: 16384,
				file: None,
			})
		);

		assert_eq!(Config::load([].into_iter()), Err(Error::MissingCommand));

		assert_eq!(
			Config::load(["-n"].into_iter().map(&str::to_string)),
			Err(Error::MissingArgument)
		);

		assert_eq!(
			Config::load(["-n", ""].into_iter().map(&str::to_string)),
			Err(Error::EmptyAction)
		);

		assert_eq!(
			Config::load(["-n", "ls -la"].into_iter().map(&str::to_string)),
			Err(Error::MissingInfoHash)
		);

		assert_eq!(
			Config::load(["-n", "ls -la", "-i"].into_iter().map(&str::to_string)),
			Err(Error::MissingArgument)
		);

		assert_eq!(
			Config::load(
				["-n", "ls -la", "-i", "abc"]
					.into_iter()
					.map(&str::to_string)
			),
			Err(Error::InvalidArgument)
		);

		assert_eq!(
			Config::load(
				["-n", "ls -la", "-i", "####"]
					.into_iter()
					.map(&str::to_string)
			),
			Err(Error::InvalidArgument)
		);

		assert_eq!(
			Config::load(
				["-n", "ls -la", "-i", "00000000000000000000000000000000000000000000000000000000000000000000000000000000", "-f", "this file doesn't exist"]
					.into_iter()
					.map(&str::to_string)
			),
			Err(Error::InvalidFile)
		);

		// directory
		assert_eq!(
			Config::load(
				["-n", "ls -la", "-i", "00000000000000000000000000000000000000000000000000000000000000000000000000000000", "-f", "src"]
					.into_iter()
					.map(&str::to_string)
			),
			Err(Error::InvalidFile)
		);
	}
}
