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

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 32;

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


fn main() {
    println!("lancement d'un client");
    let mut client = TcpStream::connect(LOCAL).expect("Le Stream n'a pas réussi à se connecter");
    client.set_nonblocking(true).expect("Echec de l'initialisation en mode non-blocking");

    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || loop {
        let mut buff = vec![0; MSG_SIZE];
        match client.read_exact(&mut buff) {
            Ok(_) => {
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                println!("Message reçu {:?}", msg);
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
                println!("message envoyé {:?}", msg);
            }, 
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break
        }

        thread::sleep(Duration::from_millis(100));
    });

    println!("Écrire un message:");
    loop {
        let mut buff = String::new();
        io::stdin().read_line(&mut buff).expect("la lecture à partir de stdin a échoué");
        let msg = buff.trim().to_string();
        if msg == ":quit" || tx.send(msg).is_err() {break}
    }
    println!("Aurevoir, à bientôt!");

}