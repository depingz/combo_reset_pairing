extern crate serialport;
use std::time::Duration;
use std::{io, thread};
use serialport::{available_ports, SerialPortType};
use serialport::prelude::*;
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
    let port_name = get_port_name();
    let mut cmd  = "261ECONF/FTR/0/FF11D80F\r\n";
    print!("{}",send_cmd(&port_name, &cmd));
    thread::sleep(Duration::from_millis(500));
    cmd = "261ECONF/FTR/0/FF16D80F\r\n";
    print!("{}",send_cmd(&port_name, &cmd));
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

    //let mut port_name = String::new();
    loop{
        print!("Please enter COM port number:");
        std::io::stdout().flush();
        let mut port_num = String::new();
        io::stdin()
            .read_line(&mut port_num)
            .expect("failed to read input.");
        port_num.trim().parse::<i32>().expect("invalid input");
        let mut port_name = "COM".to_owned() + &port_num.to_string();
        let len = &port_name.len();
        &port_name.truncate(len - 2);
        if ! port_list.contains(&port_name) {
            println!("The port number {} is invalid!", &port_name);
        }else{
            println!("Trying with {}", &port_name);
            return port_name;
        }
    }
}

fn send_cmd(port_name: &str, cmd: &str) -> String{
    let settings = SerialPortSettings {
        baud_rate: 115200,
        data_bits: DataBits::Eight,
        flow_control: FlowControl::None,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::from_millis(1000),
    };

    match serialport::open_with_settings(&port_name, & settings){
        Ok(mut port) => {
            match port.write_all(cmd.as_bytes()){
                Ok(_) =>{
                    let mut serial_buf: Vec<u8> = vec![0; 32];
                    match port.read(&mut serial_buf){
                        Ok(_) =>{
                            let readout = String::from_utf8(serial_buf).unwrap_or("Found invalid UTF-8\n".to_string());
                            let readout = readout.trim_matches(char::from(0));
                            return readout.to_string();
                        },
                        Err(_e) => return "Error read data from port!\n".to_string(),
                    }
                },
                Err(_e) => return "Error write commands to port!\n".to_string(),
            }
        },
        Err(e) => {
        e.to_string()
        },
    }
}



