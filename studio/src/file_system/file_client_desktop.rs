use {
    crate::{
        makepad_micro_serde::*,
        makepad_platform::*,
        makepad_file_protocol::{FileRequest, FileClientAction},
        makepad_file_server::{FileServerConnection, FileServer},
    },
    std::{
        //env,
        io::{Read, Write},
        net::{ TcpStream},
        sync::mpsc::{self, Receiver, Sender, TryRecvError},
        thread,
        path::Path,
        //path::PathBuf
    },
};

#[derive(Default)]
pub struct FileClient {
//    bind: Option<String>,
    //path: String,
    inner: Option<FileClientInner>
}

pub struct FileClientInner {
    pub request_sender: Sender<FileRequest>,
    pub action_signal: Signal,
    pub action_receiver: Receiver<FileClientAction>,
}

impl FileClient {
    pub fn init(&mut self, _cx:&mut Cx, path:&Path){
        if self.inner.is_none() {
            self.inner = Some(FileClientInner::new_with_local_server(path))
        }
    }
    
    pub fn send_request(&mut self, request: FileRequest) {
        self.inner.as_ref().unwrap().request_sender.send(request).unwrap();
    }
    
    pub fn request_sender(&mut self) -> impl FnMut(FileRequest) + '_ {
        let request_sender = &self.inner.as_ref().unwrap().request_sender;
        move | request | request_sender.send(request).unwrap()
    }
    
    pub fn handle_event(&mut self, cx: &mut Cx, event: &Event) -> Vec<FileClientAction> {
        let mut a = Vec::new();
        self.handle_event_with(cx, event, &mut | _, v | a.push(v));
        a
    }
    
    pub fn handle_event_with(&mut self, cx: &mut Cx, event: &Event, dispatch_action: &mut dyn FnMut(&mut Cx, FileClientAction)) {
        let inner = self.inner.as_ref().unwrap();
        match event {
            Event::Signal=>{
                loop {
                    match inner.action_receiver.try_recv() {
                        Ok(action) => dispatch_action(cx, action),
                        Err(TryRecvError::Empty) => break,
                        _ => panic!(),
                    }
                }
            }
            _ => {}
        }
    }
    
}

impl FileClientInner {
    pub fn new_with_local_server(path:&Path) -> Self {
        let (request_sender, request_receiver) = mpsc::channel();
        let action_signal = Signal::new();
        let (action_sender, action_receiver) = mpsc::channel();
        
        /*let mut root = "./".to_string();
        for arg in std::env::args(){
            if let Some(prefix) = arg.strip_prefix("--root="){
                root = prefix.to_string();
                break;
            }
        }

        let base_path = env::current_dir().unwrap().join(root);
        let final_path = base_path.join(subdir.split('/').collect::<PathBuf>());*/
        
        let mut server = FileServer::new(path);
        spawn_local_request_handler(
            request_receiver,
            server.connect(Box::new({
                let action_sender = action_sender.clone();
                let action_signal = action_signal.clone();
                move | notification | {
                    action_sender.send(FileClientAction::Notification(notification)).unwrap();
                    action_signal.set();
                }
            })),
            action_signal.clone(),
            action_sender,
        );
        //spawn_connection_listener(TcpListener::bind("127.0.0.1:0").unwrap(), server);
        
        Self {
            request_sender,
            action_signal,
            action_receiver
        }
    }
    
    pub fn new_connect_remote(to_server: &str) -> Self {
        let (request_sender, request_receiver) = mpsc::channel();
        let action_signal = Signal::new();
        let (action_sender, action_receiver) = mpsc::channel();
        
        let stream = TcpStream::connect(to_server).unwrap();
        spawn_request_sender(request_receiver, stream.try_clone().unwrap());
        spawn_response_or_notification_receiver(stream, action_signal.clone(), action_sender,);
        
        Self {
            request_sender,
            action_signal,
            action_receiver
        }
    }
    
}
/*
fn _spawn_connection_listener(listener: TcpListener, mut server: FileServer) {
    thread::spawn(move || {
        log!("Server listening on {}", listener.local_addr().unwrap());
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            log!("Incoming connection from {}", stream.peer_addr().unwrap());
            let (action_sender, action_receiver) = mpsc::channel();
            let _connection = server.connect(Box::new({
                let action_sender = action_sender.clone();
                move | notification | {
                    action_sender.send(FileClientAction::Notification(notification)).unwrap();
                }
            }));
            spawn_remote_request_handler(
                connection,
                stream.try_clone().unwrap(),
                action_sender,
            );
            spawn_response_or_notification_sender(action_receiver, stream);
        }
    });
}*/

fn _spawn_remote_request_handler(
    connection: FileServerConnection,
    mut stream: TcpStream,
    action_sender: Sender<FileClientAction>,
) {
    thread::spawn(move || loop {
        let mut len_bytes = [0; 4];
        stream.read_exact(&mut len_bytes).unwrap();
        let len = u32::from_be_bytes(len_bytes);
        let mut request_bytes = vec![0; len as usize];
        stream.read_exact(&mut request_bytes).unwrap();
        
        let request = DeBin::deserialize_bin(request_bytes.as_slice()).unwrap();
        let response = connection.handle_request(request);
        action_sender.send(FileClientAction::Response(response)).unwrap();
    });
}

fn _spawn_response_or_notification_sender(
    action_receiver: Receiver<FileClientAction>,
    mut stream: TcpStream,
) {
    thread::spawn(move || loop {
        let action = action_receiver.recv().unwrap();
        let mut action_bytes = Vec::new();
        
        action.ser_bin(&mut action_bytes);
        
        let len_bytes = action_bytes.len().to_be_bytes();
        stream.write_all(&len_bytes).unwrap();
        stream.write_all(&action_bytes).unwrap();
    });
}

fn spawn_request_sender(request_receiver: Receiver<FileRequest>, mut stream: TcpStream) {
    thread::spawn(move || loop {
        let request = request_receiver.recv().unwrap();
        let mut request_bytes = Vec::new();
        request.ser_bin(&mut request_bytes);
        let len_bytes = request_bytes.len().to_be_bytes();
        stream.write_all(&len_bytes).unwrap();
        stream.write_all(&request_bytes).unwrap();
    });
}

fn spawn_response_or_notification_receiver(
    mut stream: TcpStream,
    action_signal: Signal,
    action_sender: Sender<FileClientAction>,
) {
    thread::spawn(move || loop {
        let mut len_bytes = [0; 4];
        stream.read_exact(&mut len_bytes).unwrap();
        
        let len = u32::from_be_bytes(len_bytes);
        let mut action_bytes = vec![0; len as usize];
        stream.read_exact(&mut action_bytes).unwrap();
        let action = DeBin::deserialize_bin(action_bytes.as_slice()).unwrap();
        action_sender.send(action).unwrap();
        action_signal.set()
    });
}

fn spawn_local_request_handler(
    request_receiver: Receiver<FileRequest>,
    connection: FileServerConnection,
    action_signal: Signal,
    action_sender: Sender<FileClientAction>,
) {
    thread::spawn(move || loop {
        let request = request_receiver.recv().unwrap();
        let response = connection.handle_request(request);
        action_sender.send(FileClientAction::Response(response)).unwrap();
        action_signal.set()
    });
}