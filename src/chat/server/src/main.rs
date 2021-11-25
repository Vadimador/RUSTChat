
use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
use magic_crypt::MagicCryptTrait;
use magic_crypt::new_magic_crypt;
use std::fs;
use sha2::{Sha256, Digest};

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 100;


struct Client(std::net::TcpStream,String); // le string et l'ip:port du stream

fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(100));
}

fn main() {
    println!("lancement d'un server");

    let server = TcpListener::bind(LOCAL).expect("Échec de la liaison bind");
    server.set_nonblocking(true).expect("Echec de l'initialisation en mode non-blocking");

    let mut clients = vec![]; // les clients


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
                        let mc = new_magic_crypt!("magickey", 256);
                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let msg = String::from_utf8(msg).expect("Message utf8 invalide");
                        let msg = mc.decrypt_base64_to_string(&msg).unwrap();
                        // -------------------------------------- test pour créer un compte
                        if msg.starts_with('!') && msg.chars().nth(1).unwrap() == '!' {
                            if msg.find("!!create") != Option::None {
                                println!("demande de création d'un nouvelle account reçu !");
                                let svec : Vec<&str> = msg.split(' ').collect();
                                let account_name = svec[1].trim().to_owned();
                                let account_mdp = svec[2].trim().to_owned();
                                // -------------------------------------------- mrjoker hasher le mot de passe

                                //--------------------------------------- fin
                                // ----------------------------------------- Ritchie vérifier que l'utilisateur n'existe pas déjà
                                let contenu = fs::read_to_string("account.txt").expect("Quelque chose s'est mal passé lors de la lecture du fichier");
                                let svec : Vec<&str> = contenu.split('\n').collect();
                                let mut existe = false;
                                //println!("{:?}",svec1);
                                for x in & svec {
                                    let exist : Vec<&str> = x.split(':').collect();
                                    //println!("{} = {}", account_name, exist[0]);
                                    if account_name.eq(&exist[0]) {
                                        println!("l'utilisateur existe déjà");
                                        existe = true;
                                        break;
                                    }
                                    
                                }

                                

                                // ----------------------------------------- fin
                                if !existe {
                                    let account_mdp = hash(&account_mdp);

                                    println!("username : {}   Mot de passe : {}",account_name,account_mdp);
                                    
                                    let mut data = fs::read_to_string("account.txt").unwrap();
                                    data.push('\n');
                                    data.push_str(account_name.trim());
                                    data.push(':');
                                    data.push_str(account_mdp.trim());
                                    
                                    fs::write("account.txt",data).expect("Impossible d'écrire dans le fichier.");
                                }
                            }
                            else if msg.find("!!connect") != Option::None {
                                let svec : Vec<&str> = msg.split(' ').collect();
                                let account_name = svec[1].trim().to_owned();
                                let account_mdp = svec[2].trim().to_owned();

                                let mut data = addr.to_string();
                                data.push(' ');
                                data.push_str(account_name.trim());
                                data.push(' ');
                                data.push_str(account_mdp.trim());

                                stx.send(data).expect("Problème concernant l'envoi sur stx");
                                
                            }
                            else {
                                println!("Commande incorrect");
                            }
                        } 
                        else{
                            // on récupère le message ici et on déchiffre
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

            for client in &mut clients {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);

                client.0.write_all(&buff).expect("Connexion interrompu.");
            }
        }

        if let Ok(msg) = srx.try_recv() {
            let svec : Vec<&str> = msg.split(' ').collect();
            let account_ip = svec[0].trim().to_owned();
            let account_name = svec[1].trim().to_owned();
            let account_mdp = svec[2].trim().to_owned();

            let account_mdp = hash(&account_mdp);
            // ---- On vérifie que les informations donnée par l'utilisateur sont correct
            let contenu = fs::read_to_string("account.txt").expect("Quelque chose s'est mal passé lors de la lecture du fichier");
            let svec : Vec<&str> = contenu.split('\n').collect();
            let mut exist = false;
            for x in & svec {
                let ctab : Vec<&str> = x.split(':').collect();
                if account_name.eq(ctab[0].trim()) {
                    if account_mdp.eq(ctab[1].trim()) {
                        println!("{} s'est connecté !",account_name);
                        exist = true;
                    }
                    break;
                }
            }

            // réponse au client en question
            for client in &mut clients {
                if account_ip.eq(&client.1) {
                    if exist {
                        let mut pseudo = String::from("!!connected ");
                        pseudo.push_str(account_name.trim());
                        send_to_client(client,pseudo);
                    }
                    else {
                        send_to_client(client,String::from("!!error"));
                    }
                    break;
                }
            }
        }
        sleep();
    }
}

fn send_to_client(client : &mut Client, msg: String) {
    let mut msg = msg.into_bytes();
    msg.resize(MSG_SIZE, 0);
    client.0.write_all(&msg).expect("erreur");
}


fn hash(s : &str) -> String {
    let mut hasher = Sha256::new();

    hasher.update(&s);

    let result = hasher.finalize();
    
    let mut temp = String::new();

    for x in result{
        temp.push_str(&x.to_string());
    }
    temp
}

#[cfg(test)]
mod test {
    #[test]
    fn testing_test(){
        assert_eq!(2 + 2,4);
    }
}