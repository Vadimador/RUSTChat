// État des demandes du prof pour le serveur :
//  - ownership : tchek
//  - borrowing : tchek
//  - des collections : theck, il y a un vecteur
//  - des tests : X
//  - de la propagation d'erreur : bof
//  - des structures : nope 
//  - des enums : ... non plus
//  - des threads : OUI !

use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;
use std::str;
use magic_crypt::MagicCryptTrait;
use magic_crypt::new_magic_crypt;

const LOCAL: &str = "127.0.0.1:6000";


// test connexion tcpStream 
#[cfg(test)]
mod tests {
    #[test]
    fn test_tcpstream() {
        use std::net::TcpStream;        
        const LOCAL: &str = "127.0.0.1:6000";
        let mut test = TcpStream::connect(LOCAL).expect("Le Stream n'a pas reussi a se connecter");
    }
}


const MSG_SIZE: usize = 100;

fn main() {
    println!("lancement d'un client");
    let mut client = TcpStream::connect(LOCAL).expect("Le Stream n'a pas réussi à se connecter");
    client.set_nonblocking(true).expect("Echec de l'initialisation en mode non-blocking");

    let (tx, rx) = mpsc::channel::<String>();
    let mut name : String = "anonymous".to_owned();
    thread::spawn(move || loop {
        let mut buff = vec![0; MSG_SIZE];
        //let name = &name;
        match client.read_exact(&mut buff) {
            Ok(_) => {
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                let msg = String::from_utf8(msg).ok().unwrap();
                
               //if msg.find(&name.clone()) == Option::None {
                    println!("{:?}",msg);
               //}
            },
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("la connexion avec le serveur a été coupée !");
                break;
            }
        }

        match rx.try_recv() {
            Ok(msg) => {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).expect("l'écriture sur le socket a échoué");
            }, 
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break
        }

        thread::sleep(Duration::from_millis(100));
    });

    println!("Écrire un message:");
    loop {
        let mut buff = String::new();
        let mc = new_magic_crypt!("magickey", 256);

        io::stdin().read_line(&mut buff).expect("la lecture à partir de stdin a échoué");
        // ----------------------------------------------- Test pour savoir les commandes
        if buff.chars().next().unwrap() == ':' {        
            if buff.find(":name") != Option::None { // =========================== changer de pseudo quand on est anonymous
                let svec : Vec<&str> = buff.split(" ").collect();
                name = svec[1].trim().to_owned();
                println!("Votre nouveau nom est : {}",name.as_str());
            }
            else if buff.find(":new_account") != Option::None{ // ================ créer un nouvel account
                let svec : Vec<&str> = buff.split(" ").collect();
                let account_name = svec[1].trim().to_owned();
                let account_mdp = svec[2].trim().to_owned();

                let mut msg : String = String::from("!!create ");
                msg.push_str(&account_name.trim());
                msg.push_str(" ");
                msg.push_str(&account_mdp.trim());
                //let msg = buff.trim().to_string();
                tx.send(msg).expect("un problème est intervenu");
            }
            else if buff.find(":connect") != Option::None{ // =================== se connecter à un compte déjà crée
                let svec : Vec<&str> = buff.split(" ").collect();
                let account_name = svec[1].trim().to_owned();
                let account_mdp = svec[2].trim().to_owned();

                let mut msg : String = String::from("!!connect ");
                msg.push_str(&account_name.trim());
                msg.push_str(" ");
                msg.push_str(&account_mdp.trim());
                //let msg = buff.trim().to_string();
                tx.send(msg).expect("un problème est intervenu");
            }
            else {
                eprintln!("/!\\ La commande proposé n'existe pas.");
            }
        }
        else {
            let mut msg : String = name.clone();
            msg.push_str(" : ");
            msg.push_str(&buff.trim());
            let ciphertext = mc.encrypt_str_to_base64(&msg);
            if ciphertext == "NWNzj3mymRC2+L9S2mhsKQ==" || tx.send(ciphertext).is_err() {break}
        }
    }
    println!("Aurevoir, à bientôt!");

}