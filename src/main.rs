extern crate serialport;
use std::error::Error;
use std::time::Duration;
use std::{io, thread};
use serialport::{available_ports, SerialPortType, DataBits, StopBits, FlowControl, Parity};
use std::str;


extern crate crossterm;
use crossterm::execute;
use crossterm::cursor;
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Print;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};
use std::io::{stdout, Write};

fn main() {

    reset_radio();
    println!("Press Q to exit, press other key to continue anther unit!");
    let mut stdout = stdout();
    //going into raw mode
    enable_raw_mode().unwrap();

    //clearing the screen, going to top left corner and printing welcoming message
    // execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0), Print(r#"ctrl + q to exit, ctrl + h to print "Hello world", alt + t to print "crossterm is cool""#))
    //         .unwrap();
    //key detection
    loop {
        //going to top left corner
        //execute!(stdout, cursor::MoveTo(0, 0)).unwrap();
        //matching the key
        match read().unwrap() {
            //i think this speaks for itself
            Event::Key(KeyEvent {
                code: KeyCode::Char('h'),
                modifiers: KeyModifiers::CONTROL,
                //clearing the screen and printing our message
            }) => execute!(stdout, Clear(ClearType::All), Print("Hello world!")).unwrap(),
            Event::Key(KeyEvent {
                code: KeyCode::Char('t'),
                modifiers: KeyModifiers::ALT,
            }) => execute!(stdout, Clear(ClearType::All), Print("crossterm is cool")).unwrap(),
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: no_modifiers,
            }) => {
                disable_raw_mode().unwrap();
                std::process::exit(0)},
            _ => {execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0)).unwrap();
                Clear(ClearType::All);
                disable_raw_mode().unwrap();                
                reset_radio();
                println!("Press Q to exit, press other key to continue!");},
        }
    }
    //disabling raw mode
    //disable_raw_mode().unwrap();     
}

fn reset_radio(){
    let mut model_type = String::new();
    loop{
        print!("Please choose reset Combo or Rodeye (C/R):");
        std::io::stdout().flush();
        let mut model_input = String::new();
        io::stdin()
            .read_line(&mut model_input)
            .expect("failed to read input.");
        if model_input.trim().eq_ignore_ascii_case("Combo") || model_input.trim().eq_ignore_ascii_case("C"){
            model_type = "Combo".to_string();
            println!("Configuration for Combo");
            break;
        }
        if model_input.trim().eq_ignore_ascii_case("Rodeye") || model_input.trim().eq_ignore_ascii_case("R"){
            model_type = "Rodeye".to_string();
            println!("Configuration for Rodeye");
            break;
        }else{
            println!("Invalid input!");
        }
    }
    
    let mut cmd_1  = String::new();
    let mut cmd_2  = String::new();
    if model_type.eq_ignore_ascii_case("Combo"){
        cmd_1 = "261ECONF/FTR/0/FF11D80F\n".to_string();
        cmd_2 = "261ECONF/FTR/0/FF16D80F\n".to_string();
    }else if model_type.eq_ignore_ascii_case("Rodeye") {
        cmd_1 = "261ECONF/FTR/0/FF11D80F\n".to_string(); //  "0/CONF/FTR/0/FF11D80F"
        cmd_2 = "261ECONF/FTR/0/FF13FFFF\n".to_string(); //  "0/CONF/FTR/0/FF13FFFF"
    }else{
        println!("Found no respetive commands!");
    }
    let port_name = get_port_name();

    let mut resp_1 = String::new();
    match send_cmd(&port_name, &cmd_1){
        Ok(feedback) => resp_1 = feedback,
        Err(e) => println!("{}", e),
    };
    thread::sleep(Duration::from_millis(500));

    let mut resp_2 = String::new();
    match send_cmd(&port_name, &cmd_2){
        Ok(feedback) => resp_2 = feedback,
        Err(e) => println!("{}", e),
    };
    if resp_1 == "1E26OK\n" && resp_2 == "1E26OK\n"{
        println!("Succeed!");
    }else{
        println!("Error resetting CT301!");
    }
}

fn get_port_name() -> String{
    let mut port_list = Vec::new();
    match available_ports() {
        Ok(ports) => {
            match ports.len() {
                0 => println!("No ports found."),
                1 => println!("Found 1 port:"),
                n => println!("Found {} ports:", n),
            };
            for p in ports {
                print!("  {}", p.port_name);
                port_list.push(p.port_name);
                match p.port_type {
                    SerialPortType::UsbPort(info) => {
                        print!("    Type: USB");
                        println!(
                            "    Product: {}",
                            info.product.as_ref().map_or("", String::as_str)
                        );
                    }
                    SerialPortType::BluetoothPort => {
                        println!("    Type: Bluetooth");
                    }
                    SerialPortType::PciPort => {
                        println!("    Type: PCI");
                    }
                    SerialPortType::Unknown => {
                        println!("    Type: Unknown");                        
                    }                    
                }
            }            
            std::io::stdout().flush();
        }
        Err(e) => {
            eprintln!("{:?}", e);
            eprintln!("Error listing serial ports");
        }
    }

    loop{
        print!("Please enter COM port number:");
        std::io::stdout().flush();
        let mut port_num = String::new();
        io::stdin()
            .read_line(&mut port_num)
            .expect("failed to read input.");
        match port_num.trim().parse::<i32>(){
            Ok(num) => {
                let mut port_name = "COM".to_owned() + &port_num.to_string();
                let len = &port_name.len();
                &port_name.truncate(len - 2);
                if ! port_list.contains(&port_name) {
                    println!("The port number {} is invalid!", &port_name);
                }else{
                    return port_name;
                }
            },
            Err(e) =>{
                println!("The port number is invalid!");
            }
        }
    }
}

fn send_cmd(port_name: &str, cmd: &str) -> Result<String, Box<dyn Error>>{
    let baud_rate = 115_200;
    let mut port = serialport::new(port_name, baud_rate)
        .data_bits(DataBits::Eight)
        .stop_bits(StopBits::One)
        .flow_control(FlowControl::None)
        .parity(Parity::None)
        .timeout(Duration::from_millis(1_000))
        .open()
        .map_err(|ref e| format!("Port '{}' not available: {}", &port_name, e))?;
    println!("Connected to {} at {} baud", &port_name, &baud_rate);

    port.write_all(cmd.as_bytes()).map_err(|ref e| format!("Error while writing data to the port: {}",e))?;
    println!("Sent data: {:?}", &cmd);

    let mut serial_buf: Vec<u8> = vec![0; 32];
    port.read(&mut serial_buf).map_err(|ref e| format!("Error while reading data from the port: {}",e))?;
    let mut feedback = String::from_utf8(serial_buf).map_err(|ref e| format!("Found invalid UTF-8: {}\n",e))?;
    feedback = feedback.trim_matches(char::from(0)).to_string();
    println!("Port response: {:?}",&feedback);
    Ok(feedback)
}



