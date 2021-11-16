// État des demandes du prof pour le serveur :
//  - ownership : tchek
//  - borrowing : tchek
//  - des collections : theck, il y a un vecteur
//  - des tests : X
//  - de la propagation d'erreur : bof
//  - des structures : X
//  - des enums : X
//  - des threads : OUI !

use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
//use std::fs::File;
use std::fs;
//use std::io::prelude::*;

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 64;

struct Client(std::net::TcpStream,String);

fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(100));
}

fn main() {
    println!("lancement d'un server");

    let server = TcpListener::bind(LOCAL).expect("Échec de la liaison bind");
    server.set_nonblocking(true).expect("Echec de l'initialisation en mode non-blocking");

    let mut clients = vec![];
    let (tx, rx) = mpsc::channel::<String>();
    loop {
        if let Ok((mut socket, addr)) = server.accept() {
            println!("Client {} connecté", addr);

            let tx = tx.clone();
            //clients.push(socket.try_clone().expect("échec du clonage du client"));
            clients.push(Client(socket.try_clone().expect("échec du clonage du client"),String::from("unknown")));

            thread::spawn(move || loop {
                let mut buff = vec![0; MSG_SIZE];

                match socket.read_exact(&mut buff) {
                    Ok(_) => {
                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let msg = String::from_utf8(msg).expect("Message utf8 invalide");
                        // -------------------------------------- test pour créer un compte
                        if msg.chars().nth(0).unwrap() == '!' && msg.chars().nth(1).unwrap() == '!' {
                            println!("demande de création d'un nouvelle account reçu !");
                            let svec : Vec<&str> = msg.split(" ").collect();
                            let account_name = svec[1].trim().to_owned();
                            let account_mdp = svec[2].trim().to_owned();
                            println!("username : {}   Mot de passe : {}",account_name,account_mdp);
                            
                            //let mut file = File::open("account.txt").unwrap();
                            //file.write_all(&data.into_bytes()).expect("Impossible d'écrire dans le fichier account.txt"); // problème de permissions
                            let mut data = fs::read_to_string("account.txt").unwrap();
                            data.push_str("\n");
                            data.push_str(account_name.trim());
                            data.push_str(":");
                            data.push_str(account_mdp.trim());
                            
                            fs::write("account.txt",data).expect("Impossible d'écrire dans le fichier.");
                           

                        } 
                        else{
                            println!("{}: {:?}", addr, msg);
                            tx.send(msg).expect("échec de l'envoi de msg à rx"); 
                        }
                       
                    }, 
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("fermeture de la connexion avec : {}", addr);
                        break;
                    }
                }

                sleep();
            });
        }

        if let Ok(msg) = rx.try_recv() {
            clients = clients.into_iter().filter_map(|mut client| {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);

                client.0.write_all(&buff).map(|_| client).ok()
            }).collect::<Vec<_>>();
        }

        sleep();
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn testing_test(){
        assert_eq!(2 + 2,4);
    }
}