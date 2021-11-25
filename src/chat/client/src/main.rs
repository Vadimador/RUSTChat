use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self};
use std::thread;
use std::str;
use magic_crypt::MagicCryptTrait;
use magic_crypt::new_magic_crypt;

const LOCAL: &str = "127.0.0.1:6000";


// test connexion tcpStream 
#[cfg(test)]
mod tests {
    #[test]
// Le test échoue lorsque le serveur est éteint
    fn test_tcpstream() {
        use std::net::TcpStream;        
        const LOCAL: &str = "127.0.0.1:6000";
        let _test = TcpStream::connect(LOCAL).expect("Le Stream n'a pas reussi a se connecter");
    }
}


#[cfg(test)]
mod test_crypto {
    #[test]
    fn test_crypto(){

        use magic_crypt::new_magic_crypt;
        use magic_crypt::MagicCryptTrait;
            
        let mc = new_magic_crypt!("magickey", 256);

        let base64 = mc.encrypt_str_to_base64("test de chiffrement");

        assert_eq!("nS5LGKNVbvoGNRFegY43NpJEsGSB8B69qNrR8dOf2Ng=", base64);

        assert_eq!("test de chiffrement", mc.decrypt_base64_to_string(&base64).unwrap());

    }
        
    
}

const MSG_SIZE: usize = 256;

enum Anon {
    Anonyme,
    Connecte,
}

fn main() {
    println!("lancement d'un client");
    let mut client = TcpStream::connect(LOCAL).expect("Le Stream n'a pas réussi à se connecter"); // connexion via tunnel en tcp (socket)
    client.set_nonblocking(true).expect("Echec de l'initialisation en mode non-blocking");

    let (tx, rx) = mpsc::channel::<String>(); // permet la communication entre tous les thread locaux.

    let mut name : String = "anonymous".to_owned();
    let mut name_connected = "✅ bruh".to_owned();
    let mut anon : Anon = Anon::Anonyme;

    println!("Écrire un message:");
    thread::spawn( move || loop {
        let mut buff = String::new();
        io::stdin().read_line(&mut buff).expect("la lecture à partir de stdin a échoué");
        tx.send(buff).expect("transmission du channel impossible");
    });

    loop {
        let mut buff = vec![0; MSG_SIZE];
        match client.read_exact(&mut buff) { // lit la socket
            Ok(_) => {
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                let msg = String::from_utf8(msg).ok().unwrap();
                
                if !msg.is_empty() {
                    if msg.starts_with('!') && msg.chars().nth(1).unwrap() == '!' {
                        if msg.find("!!connected") != Option::None { // lorsque l'utilisateur se connecte à un compte
                            name_connected = String::from("✅ ");
                            let svec : Vec<&str> = msg.split(' ').collect();
                            name_connected.push_str(svec[1]);
                            anon =  Anon::Connecte;
                            println!("Vous êtes connecté en tant que \"{}\"",svec[1]);
                        }
                        else if msg.find("!!error") != Option::None { // si il fail
                            println!("Le nom d'utilisateur ou le mot de passe est incorrect.");
                        }
                    }
                    else {
                        println!("{:?}",msg);
                    } 
                 }
                
            },
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("la connexion avec le serveur a été coupée !");
                break;
            }
        }

        if let Ok(buff) = rx.try_recv() { // reçois les entrées utilisateurs
            // ----------------------------------------------- Test pour savoir les commandes
            if buff.starts_with(':') {        
                if buff.find(":name") != Option::None { // =========================== changer de pseudo quand on est anonymous
                    let svec : Vec<&str> = buff.split(' ').collect();
                    if svec.len() == 2 {
                        
                        if !svec[1].trim().starts_with('✅') {
                            name = svec[1].trim().to_owned();
                            println!("Votre nouveau nom d'anonyme est : {}",name.as_str());
                        }
                        else {
                            println!("Impossible d'avoir ce nom là");
                        }
                    }
                    else {
                        println!("erreur dans le nombre d'argument, tapez \":help\" pour plus d'information");
                    }
                }
                else if buff.find(":new_account") != Option::None{ // ================ créer un nouvel account
                        let svec : Vec<&str> = buff.split(' ').collect();
                    if svec.len() == 3 {
                        let account_name = svec[1].trim().to_owned();
                        let account_mdp = svec[2].trim().to_owned();

                        let mut msg : String = String::from("!!create ");
                        msg.push_str(account_name.trim());
                        msg.push(' ');
                        msg.push_str(account_mdp.trim());
                        send(&mut client, msg);
                    }
                    else {
                        println!("erreur dans le nombre d'argument, tapez \":help\" pour plus d'information.");
                    }
                }
                else if buff.find(":connect") != Option::None{ // =================== se connecter à un compte déjà crée
                    let svec : Vec<&str> = buff.split(' ').collect();
                    if svec.len() == 3 {
                        let account_name = svec[1].trim().to_owned();
                        let account_mdp = svec[2].trim().to_owned();

                        let mut msg : String = String::from("!!connect ");
                        msg.push_str(account_name.trim());
                        msg.push(' ');
                        msg.push_str(account_mdp.trim());
                        send(&mut client, msg);
                    }
                    else {
                        println!("erreur dans le nombre d'argument, tapez \":help\" pour plus d'information.");
                    }
                }
                else if buff.find(":quit") != Option::None {
                    match anon {
                        Anon::Anonyme => {
                            break;
                        },
                        Anon::Connecte => {
                            anon = Anon::Anonyme;
                            println!("Vous vous êtes déconnecté.");
                        }
                    }
                    
                }
                else if buff.find(":help") != Option::None {
                    println!("Commandes existantes : ");
                    println!("  :name <pseudo> --> change votre pseudo d'anonyme.");
                    println!("  :new_account <pseudo> <mot de passe> --> créer un nouveau compte.");
                    println!("  :connect <pseudo> <mot de passe> --> vous connecte à un compte déjà créer.");
                    println!("  :quit --> vous déconnecte d'un compte ou ferme la connexion distante.");
                }
                else {
                    eprintln!("/!\\ La commande proposé n'existe pas.");
                }
            }
            else {
                let mut msg : String;
                match anon {
                    Anon::Anonyme => {
                        msg = name.clone();
                    },
                    Anon::Connecte => {
                        msg = name_connected.clone();
                    }
                }
                    
                msg.push_str(" : ");
                msg.push_str(buff.trim());
               
                send(&mut client,msg);
            }
        }
    }
    println!("Aurevoir, à bientôt!");

}

fn send(client : &mut std::net::TcpStream, msg : String) {
    let mc = new_magic_crypt!("magickey", 256);
    let msg = mc.encrypt_str_to_base64(&msg);
    let mut msg = msg.into_bytes();
    msg.resize(MSG_SIZE, 0);
    client.write_all(&msg).expect("l'écriture sur le socket a échoué");
}