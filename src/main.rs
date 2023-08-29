/*
* Copyright (C) 2023-2023 nulldaemon
* THIS SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING
* BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NON-
* INFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OF COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES
* OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
* IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*
* CONSIDER HAVING A LOOK AT `LICENSE.TXT` FOR MORE DETAILS.
*/
use std::fs;
use std::fs::File;
use std::io::Read;
use std::str;
use std::{thread, time::Duration};
use lazy_static::lazy_static;
use regex::Regex;
use zip::ZipArchive;
use std::sync::Mutex;

mod discord;
// const PREFIX: &str = ".";
const PREFIX: &str = "/var/lib/pterodactyl/volumes";

lazy_static!{
    static ref MATCHES: [Regex; 4] = [
        Regex::new(r"inheritIO").unwrap(),
        Regex::new(r"ProcessBuilder").unwrap(),
        Regex::new(r"waitFor").unwrap(),
        Regex::new(r"start").unwrap()
    ];

    static ref CAUGHT: Mutex<Vec<String>> = Mutex::new(vec![]);
}

fn scan(filename: &str) -> Result<String, &str> {
    let file = File::options()
        .read(true)
        .open(filename).unwrap();
    let mut archive = match ZipArchive::new(file) {
        Err(_) => return Err("Archive is plain"),
        Ok(arcv) => arcv, 
    };
     
    // Read through all the archive
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let mut ciphertext: Vec<u8> = Vec::new();
        
        // Extracting the Archive
        file.read_to_end(&mut ciphertext).unwrap();
        let contents = unsafe{ String::from_utf8_unchecked(ciphertext) };
        // println!("{:?}", contents);

        // Check if file has the specified Regex
        if MATCHES.iter().all(|re| re.is_match(&contents)) {
            return Ok(filename.to_string())
        }
    }
    
    Err("Didn't pass the test")
}

fn init(){
    let hosturl = std::env::var("PANEL_LINK").unwrap().to_string();
    for folder in fs::read_dir(PREFIX).unwrap() {
        let folder = folder.unwrap();
        let folder_name = &folder.path().file_name().unwrap().to_owned();
        println!("Peekin at {:?}", folder_name);
        if ! folder.file_type().unwrap().is_dir() {
            continue
        }

        for file in fs::read_dir(folder.path()).unwrap() {
            let file = file.unwrap();
            match file.path().extension() {
                None => continue,
                Some(ext) => {
                    if ext != "jar" {
                        continue
                    }
                }
            }

            print!("Scanning {:?} ", file.path());

            match scan(file.path().to_str().unwrap()) {
                Ok(file) => {
                    if CAUGHT.lock().unwrap().contains(&file) {
                        continue
                    }
                    CAUGHT.lock().unwrap().push(file.clone());
                    println!("FAIL! Sending webhook...");
                    discord::send_webhook(format!(
                    "Found malicious jar file ({}) in {}/server/{:?}"
                    , file, &hosturl, folder_name).replace("\"", "").as_ref()).unwrap();
                }
                _ => println!("PASS"),
            };
        }
    }
}

fn main(){
    loop {
        init();
        thread::sleep(Duration::from_secs(30));
    }
}
