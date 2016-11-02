extern crate crypto;

use utils;
use errors;
use models;

use self::crypto::md5::Md5;
use self::crypto::digest::Digest;

use std::net::{SocketAddrV4, TcpStream, Ipv4Addr};
use std::io::{self, Read, Write};
use std::str::FromStr;
use std::collections::BTreeMap;

pub const DEFAULT_PORT: u16 = 8728;

pub struct Client {
    sock_addr: SocketAddrV4,
    stream: TcpStream,
}

impl Client {
    pub fn connect(ip: &str, port: &str) -> Result<Client, errors::CreateClientError> {
        let ip = try!(Ipv4Addr::from_str(ip));
        let port = port.parse::<u16>().unwrap_or(DEFAULT_PORT);
        let sock_addr = SocketAddrV4::new(ip, port);

        println!("Connecting to {}:{}...", ip, port);
        let stream = try!(TcpStream::connect(sock_addr));

        Ok(Client {
            sock_addr: sock_addr,
            stream: stream,
        })
    }

    pub fn print(&self) {
        println!("Router OS Client.");
        println!("Connecting to server {}:{}",
                 self.sock_addr.ip(),
                 self.sock_addr.port());
    }

    fn read_str(&mut self, total_bytes: usize) -> Result<Vec<u8>, io::Error> {
        let mut buffer: Vec<u8> = vec![0; total_bytes];
        let mut bytes_read = 0 as usize;

        while bytes_read < total_bytes {
            let x = try!(self.stream.read(&mut buffer[bytes_read..]));

            if x == 0 {
                let err = io::Error::new(io::ErrorKind::ConnectionReset,
                                         "error reading: connection closed by remote end");
                return Err(err);
            }

            bytes_read += x;
        }

        Ok(buffer)
    }

    fn write_str(&mut self, buffer: &[u8]) -> Result<(), io::Error> {
        let total_bytes = buffer.len();
        let mut bytes_written = 0;

        while bytes_written < total_bytes {
            let x = try!(self.stream.write(&buffer[bytes_written..]));

            if x == 0 {
                let err = io::Error::new(io::ErrorKind::ConnectionReset,
                                         "error writing: connection closed by remote end");
                return Err(err);
            }

            bytes_written += x;
        }

        Ok(())
    }

    fn read_len(&mut self) -> Result<usize, io::Error> {
        let mut c = try!(self.read_str(1))[0] as usize;

        if c & 0x80 == 0x00 {
            return Ok(c);
        } else if c & 0xC0 == 0x80 {
            c &= !0xC0;
            c << 8;
            c += try!(self.read_str(1))[0] as usize;
        } else if c & 0xE0 == 0xC0 {
            c &= !0xE0;
            c << 8;
            c += try!(self.read_str(1))[0] as usize;
            c << 8;
            c += try!(self.read_str(1))[0] as usize;
        } else if c & 0xF0 == 0xE0 {
            c &= !0xF0;
            c << 8;
            c += try!(self.read_str(1))[0] as usize;
            c << 8;
            c += try!(self.read_str(1))[0] as usize;
            c << 8;
            c += try!(self.read_str(1))[0] as usize;
        } else if c & 0xF8 == 0xF0 {
            c += try!(self.read_str(1))[0] as usize;
            c << 8;
            c += try!(self.read_str(1))[0] as usize;
            c << 8;
            c += try!(self.read_str(1))[0] as usize;
            c << 8;
            c += try!(self.read_str(1))[0] as usize;
        }

        Ok(c)
    }

    fn write_len(&mut self, mut len: usize) -> Result<(), io::Error> {
        if len < 0x80 {
            try!(self.write_str(&[len as u8]));
        } else if len < 0x4000 {
            len |= 0x8000;
            try!(self.write_str(&[((len >> 8) & 0xFF) as u8]));
            try!(self.write_str(&[(len & 0xFF) as u8]));
        } else if len < 0x200000 {
            len |= 0xC00000;
            try!(self.write_str(&[((len >> 16) & 0xFF) as u8]));
            try!(self.write_str(&[((len >> 8) & 0xFF) as u8]));
            try!(self.write_str(&[(len & 0xFF) as u8]));
        } else if len < 0x10000000 {
            len |= 0xE0000000;
            try!(self.write_str(&[((len >> 24) & 0xFF) as u8]));
            try!(self.write_str(&[((len >> 16) & 0xFF) as u8]));
            try!(self.write_str(&[((len >> 8) & 0xFF) as u8]));
            try!(self.write_str(&[(len & 0xFF) as u8]));
        } else {
            try!(self.write_str(&[0xF0 as u8]));
            try!(self.write_str(&[((len >> 24) & 0xFF) as u8]));
            try!(self.write_str(&[((len >> 16) & 0xFF) as u8]));
            try!(self.write_str(&[((len >> 8) & 0xFF) as u8]));
            try!(self.write_str(&[(len & 0xFF) as u8]));
        }

        Ok(())
    }

    fn read_word(&mut self) -> Result<String, io::Error> {
        let str_len = try!(self.read_len());
        let bytes = try!(self.read_str(str_len));
        let result: String = (&bytes).into_iter().map(|x| *x as char).collect();

        Ok(result)
    }

    fn write_word(&mut self, word: &String) -> Result<(), io::Error> {
        try!(self.write_len(word.len()));
        try!(self.write_str(word.as_bytes()));

        Ok(())
    }

    pub fn write_sentence(&mut self, words: &Vec<String>) -> Result<(), io::Error> {
        for w in words.iter() {
            try!(self.write_word(&w));
        }

        try!(self.write_word(&String::from("")));

        Ok(())
    }

    pub fn read_sentence(&mut self) -> Result<Vec<String>, io::Error> {
        let mut words: Vec<String> = Vec::new();

        loop {
            let w = try!(self.read_word());

            if w.is_empty() {
                return Ok(words);
            }

            words.push(w);
        }
    }

    fn is_mk_error(&self, sentence: &Vec<String>) -> Option<errors::MikrotikError> {
        if sentence.len() == 0 {
            return None;
        }

        match sentence[0].as_str() {
            "!trap" => {
                let mut category: u8 = 10;
                let mut msg = String::new();

                if sentence.len() < 1 {
                    // No category or message supplied.
                    return Some(errors::MikrotikError::Trap {
                        category: category,
                        msg: msg,
                    });
                }

                for word in &sentence[1..] {
                    if word.starts_with("=category=") {
                        let v: Vec<&str> = word.split("=category=").collect();
                        category = v[1].parse::<u8>().unwrap_or(category);
                    } else if word.starts_with("=message=") {
                        let v: Vec<&str> = word.split("=message=").collect();
                        msg.push_str(v[1]);
                    }
                }

                return Some(errors::MikrotikError::Trap {
                    category: category,
                    msg: msg,
                });
            }
            "!fatal" => {
                if sentence.len() < 1 {
                    let msg = String::from("No error message supplied from router!");
                    return Some(errors::MikrotikError::Fatal(msg));
                }

                let msg = String::from(sentence[1].as_str());
                return Some(errors::MikrotikError::Fatal(msg));
            }
            _ => return None,
        }
    }

    pub fn read_all_sentences(&mut self) -> Result<Vec<String>, errors::MikrotikError> {
        let mut sentences: Vec<String> = Vec::new();

        loop {
            let sentence = try!(self.read_sentence());

            if sentence.len() == 0 {
                continue;
            }

            if let Some(e) = self.is_mk_error(&sentence) {
                return Err(e);
            }

            sentences.extend_from_slice(&sentence);

            if sentence[0] == "!done" {
                break;
            }
        }

        Ok(sentences)
    }

    fn execute(&mut self, sentence: &Vec<String>) -> Result<Vec<String>, errors::MikrotikError> {
        if sentence.len() == 0 {
            return Ok(vec![]);
        }

        try!(self.write_sentence(sentence));

        let response = try!(self.read_all_sentences());
        if response.len() == 0 {
            return Ok(vec![]);
        }

        Ok(response)
    }

    pub fn login(&mut self, username: &str, pwd: &str) -> Result<(), errors::MikrotikError> {
        let mut login_request = vec!["/login".to_string()];
        let response = try!(self.execute(&login_request));

        let hexstr: Vec<&str> = response[1].split("=ret=").collect();
        let challenge = try!(utils::unhexlify(hexstr[1]));

        let mut md = Md5::new();
        md.input(&[0]);
        md.input(pwd.as_bytes());
        md.input(challenge.as_slice());

        login_request.push(format!("=name={}", username));
        login_request.push(format!("=response=00{}", md.result_str()));

        try!(self.execute(&login_request));

        Ok(())
    }

    pub fn get_address_list(&mut self) -> Result<Vec<models::IPAddress>, errors::MikrotikError> {
        let request = vec!["/ip/firewall/address-list/print".to_string()];
        let response = try!(self.execute(&request));

        let mut addresses: Vec<models::IPAddress> = Vec::new();
        let mut address = models::IPAddress::new();
        let mut is_first = true;

        for word in response {
            if word.starts_with("!re") {
                if is_first {
                    is_first = false;
                    continue;
                }

                addresses.push(address.clone());
                address = models::IPAddress::new();
            } else if word.starts_with("=.id=") {
                let v: Vec<&str> = word.split("=.id=").collect();
                address.id.push_str(v[1]);
            } else if word.starts_with("=list=") {
                let v: Vec<&str> = word.split("=list=").collect();
                address.list.push_str(v[1]);
            } else if word.starts_with("=address=") {
                let v: Vec<&str> = word.split("=address=").collect();
                address.address.push_str(v[1]);
            } else if word.starts_with("!done") {
                addresses.push(address.clone());
                break;
            }
        }

        Ok(addresses)
    }

    pub fn get_queue_list(&mut self) -> Result<Vec<models::Client>, errors::MikrotikError> {
        let request = vec!["/queue/simple/print".to_string()];
        let response = try!(self.execute(&request));

        let mut clients: Vec<models::Client> = Vec::new();
        let mut client = models::Client::new();
        let mut is_first = true;

        for word in response {
            if word.starts_with("!re") {
                if is_first {
                    is_first = false;
                    continue;
                }

                clients.push(client.clone());
                client = models::Client::new();
            } else if word.starts_with("=.id=") {
                let v: Vec<&str> = word.split("=.id=").collect();
                client.id.push_str(v[1]);
            } else if word.starts_with("=name=") {
                let v: Vec<&str> = word.split("=name=").collect();
                client.name.push_str(v[1]);
            } else if word.starts_with("=target=") {
                let v: Vec<&str> = word.split("=target=").collect();
                client.target.push_str(v[1]);
            } else if word.starts_with("=max-limit=") {
                let v: Vec<&str> = word.split("=max-limit=").collect();
                client.max_limit.push_str(v[1]);
            } else if word.starts_with("=burst-limit=") {
                let v: Vec<&str> = word.split("=burst-limit=").collect();
                client.burst_limit.push_str(v[1]);
            } else if word.starts_with("=burst-threshold=") {
                let v: Vec<&str> = word.split("=burst-threshold=").collect();
                client.burst_threshold.push_str(v[1]);
            } else if word.starts_with("=burst-time=") {
                let v: Vec<&str> = word.split("=burst-time=").collect();
                client.burst_time.push_str(v[1]);
            } else if word.starts_with("!done") {
                clients.push(client.clone());
                break;
            }
        }

        Ok(clients)
    }

    fn talk(&mut self,
            words: &Vec<String>)
            -> Result<Vec<(String, BTreeMap<String, String>)>, io::Error> {
        if words.len() == 0 {
            return Ok(vec![]);
        }

        try!(self.write_sentence(words));
        let mut r: Vec<(String, BTreeMap<String, String>)> = Vec::new();

        loop {
            let sentence = try!(self.read_sentence());
            if sentence.len() == 0 {
                continue;
            }

            let reply = &sentence[0];
            let mut attrs: BTreeMap<String, String> = BTreeMap::new();

            for word in &sentence[1..] {
                match word[1..].find("=") {
                    Some(n) => {
                        attrs.insert(word[..n + 1].to_string(), word[(n + 2)..].to_string());
                    }
                    None => {
                        attrs.insert(word.clone(), String::from(""));
                    }
                }
            }

            r.push((reply.clone(), attrs));
            if reply == "!done" {
                return Ok(r);
            }
        }
    }

    pub fn old_login(&mut self, username: &str, pwd: &str) -> Result<(), io::Error> {
        let mut challenge: Vec<u8> = Vec::new();
        let mut login_request = vec![r"/login".to_string()];

        for (_, attrs) in try!(self.talk(&login_request)) {
            challenge = utils::unhexlify(&attrs["=ret"]).unwrap();
        }

        let mut md = Md5::new();
        md.input(&[0]);
        md.input(pwd.as_bytes());
        md.input(&challenge[..]);

        login_request.push(format!("=name={}", username));
        login_request.push(format!("=response=00{}", md.result_str()));

        try!(self.talk(&login_request));

        Ok(())
    }
}
