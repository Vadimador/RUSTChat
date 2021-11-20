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
use magic_crypt::MagicCryptTrait;
use magic_crypt::new_magic_crypt;

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 100;



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
            clients.push(socket.try_clone().expect("échec du clonage du client"));

            thread::spawn(move || loop {
                let mut buff = vec![0; MSG_SIZE];

                match socket.read_exact(&mut buff) {
                    Ok(_) => {
                        // on récupère le message ici et on déchiffre
                        let mc = new_magic_crypt!("magickey", 256);
                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let msg = String::from_utf8(msg).expect("Message utf8 invalide");
                        let msg = mc.decrypt_base64_to_string(&msg).unwrap();

                        println!("{}: {:?}", addr, msg);
                        tx.send(msg).expect("échec de l'envoi de msg à rx");
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

                client.write_all(&buff).map(|_| client).ok()
            }).collect::<Vec<_>>();
        }

        sleep();
    }
}