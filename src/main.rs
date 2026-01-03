use std::{fs::File, io::Read};

fn get_lines(mut file: File) -> Vec<String> {
    let mut buffer = [0u8; 8];
    let mut lines = Vec::new();

    let mut line = String::new();
    loop {
        let bytes_read = file
            .read(&mut buffer)
            .expect("Failed to read the file contents!");

        if bytes_read == 0 {
            break;
        }

        line += str::from_utf8(&buffer[..bytes_read]).expect("Failed to convert to String");
        let split_lines: Vec<&str> = line.split('\n').collect();

        if split_lines.len() > 1 {
            lines.push(split_lines[0].to_string());
            line = String::from(split_lines[1].to_string());
        } else {
            line = String::from(split_lines[0]);
        }
    }
    if line.len() != 0 {
        lines.push(line.to_string());
    }

    lines
}

fn main() {
    let file = File::open("messages.txt").expect("Failed to read the file!");

    let lines = get_lines(file);
    for line in lines {
        println!("read: {}", line);
    }
}
