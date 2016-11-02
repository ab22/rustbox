extern crate mikrotik;

use std::env;


// fn print_usage() {
// println!("USAGE: rustbox [server ip] [user] [password] [port]?");
// }
//
// fn get_line() -> String {
// let mut input = String::new();
// io::stdin().read_line(&mut input).unwrap();
//
// input.pop();
// input
// }

fn main() {
    let mut args = env::args();
    let ip = args.nth(1).expect("No server ip specified!");
    let user = args.nth(0).expect("No user specified!");
    let pw = args.nth(0).expect("No password specified!");
    let port = args.nth(0).unwrap_or(String::new());

    let mut mtclient = mikrotik::Client::connect(&ip, &port).unwrap();
    println!("Connected!");

    println!["Performing login..."];
    mtclient.login(&user, &pw).unwrap();
    mtclient.print();
    println!("Connected!!");

    // let addresses = mtclient.get_address_list().unwrap();
    // for address in addresses {
    // println!("[ID: {}] - [{}] - [{}]",
    // address.id,
    // address.address,
    // address.list);
    // }

    let clients = mtclient.get_queue_list().unwrap();
    for client in clients {
        println!("[{}] - [{}] - [{}]", client.id, client.name, client.target);
        println!("[{}] - [{}] - [{}] - [{}]",
                 client.max_limit,
                 client.burst_limit,
                 client.burst_threshold,
                 client.burst_time);
    }

    // 'main_loop: loop {
    // if has_written {
    // 'reply_loop: loop {
    // let replies = mtclient.read_sentence().unwrap();
    // if replies.len() == 0 {
    // continue;
    // }
    // if replies[0] == "!done" {
    // has_written = false;
    // break 'reply_loop;
    // } else {
    // for reply in replies {
    // println!("> {}", reply);
    // }
    // }
    // }
    // } else {
    // let input = get_line();
    // if &input[..] == "#quit#" {
    // break 'main_loop;
    // }
    //
    // if &input[..] == "" && was_command {
    // mtclient.write_sentence(&input_sentence).unwrap();
    // input_sentence.clear();
    // was_command = false;
    // has_written = true;
    // } else {
    // input_sentence.push(input);
    // was_command = true;
    // has_written = false;
    // }
    // }
    // }


    println!("Good bye!");
}
