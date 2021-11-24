// État des demandes du prof pour le serveur :
//  - ownership : OUI
//  - borrowing : OUI
//  - des collections : theck, il y a un vecteur
//  - des tests : X
//  - de la propagation d'erreur : bof
//  - des structures : OUI
//  - des enums : X
//  - des threads : OUI !

use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
use magic_crypt::MagicCryptTrait;
use magic_crypt::new_magic_crypt;
use std::fs;
use std::collections::HashMap;

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 100;


struct Client(std::net::TcpStream,String); // le sting et l'ip:port du stream

fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(100));
}

fn main() {
    println!("lancement d'un server");

    let server = TcpListener::bind(LOCAL).expect("Échec de la liaison bind");
    server.set_nonblocking(true).expect("Echec de l'initialisation en mode non-blocking");

    let mut clients = vec![]; // les clients
    let mut hashclients = HashMap::<String,String>::new();

    let (tx, rx) = mpsc::channel::<String>(); // channel principal commun
    let (stx, srx) = mpsc::channel::<String>(); // channel spécial pour les changements de compte
    loop {
        if let Ok((mut socket, addr)) = server.accept() {
            println!("Client {} connecté", addr);

            let tx = tx.clone();
            let stx = stx.clone();

            clients.push(Client(socket.try_clone().expect("échec du clonage du client"),addr.to_string()));

            thread::spawn(move || loop {
                let mut buff = vec![0; MSG_SIZE];

                match socket.read_exact(&mut buff) {
                    Ok(_) => {
                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let msg = String::from_utf8(msg).expect("Message utf8 invalide");
                        // -------------------------------------- test pour créer un compte
                        if msg.chars().nth(0).unwrap() == '!' && msg.chars().nth(1).unwrap() == '!' {
                            if msg.find("!!create") != Option::None {
                                println!("demande de création d'un nouvelle account reçu !");
                                let svec : Vec<&str> = msg.split(" ").collect();
                                let account_name = svec[1].trim().to_owned();
                                let account_mdp = svec[2].trim().to_owned();
                                // -------------------------------------------- mrjoker hasher le mot de passe
                                


                                //--------------------------------------- fin
                                // ----------------------------------------- Ritchie vérifier que l'utilisateur n'existe pas déjà


                                // ----------------------------------------- fin
                                println!("username : {}   Mot de passe : {}",account_name,account_mdp);
                                
                                 let mut data = fs::read_to_string("account.txt").unwrap();
                                data.push_str("\n");
                                data.push_str(account_name.trim());
                                data.push_str(":");
                                data.push_str(account_mdp.trim());
                                
                                fs::write("account.txt",data).expect("Impossible d'écrire dans le fichier.");
                            }
                            else if msg.find("!!connect") != Option::None {
                                let svec : Vec<&str> = msg.split(" ").collect();
                                let account_name = svec[1].trim().to_owned();
                                let account_mdp = svec[2].trim().to_owned();

                                let mut data = addr.to_string();
                                data.push_str(" ");
                                data.push_str(account_name.trim());
                                data.push_str(" ");
                                data.push_str(account_mdp.trim());

                                stx.send(data).expect("Problème concernant l'envoi sur stx");
                                
                            }
                            else {
                                println!("Commande incorrect");
                            }
                        } 
                        else{
                            // on récupère le message ici et on déchiffre
                            let mc = new_magic_crypt!("magickey", 256);
                            let msg = mc.decrypt_base64_to_string(&msg).unwrap();
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
            //let c = clients.clone();
            clients = clients.into_iter().filter_map(|mut client| {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);

                client.0.write_all(&buff).map(|_| client).ok()
                
            }).collect::<Vec<_>>();
            
            /*for mut c in &clients {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);

                c.0.write_all(&buff);
            }*/
        }

        if let Ok(msg) = srx.try_recv() {
            let svec : Vec<&str> = msg.split(" ").collect();
            let account_ip = svec[0].trim().to_owned();
            let account_name = svec[1].trim().to_owned();
            let account_mdp = svec[2].trim().to_owned();
            // -------------------------------- mrjoker hash le mdp reçu et tester la validité des informations



            for client in &mut clients {
                if account_ip.eq(&client.1) {
                    let mut buff2 = String::from("!!connected pseudo").into_bytes();
                    buff2.resize(MSG_SIZE, 0);
                    //let monclient = &mut client.0;
                    client.0.write_all(&buff2).expect("erreur");
                    break;
                }
            }
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