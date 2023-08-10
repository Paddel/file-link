const configuration = { iceServers: [{ urls: 'stun:stun.l.google.com:19302' }] };
const peerConnection = new RTCPeerConnection(configuration);

const fileInput = document.getElementById('fileInput');
const startButton = document.getElementById('startButton');
const sendFileButton = document.getElementById('sendFile');
const connectionStatus = document.getElementById('connectionStatus');

const ws = new WebSocket('ws://127.0.0.1:9000');

// Handle incoming signaling messages
ws.onmessage = (event) => {
  const message = JSON.parse(event.data);

  switch (message.type) {
    case 'offer':
      peerConnection.setRemoteDescription(new RTCSessionDescription(message.offer))
        .then(() => peerConnection.createAnswer())
        .then(answer => peerConnection.setLocalDescription(answer))
        .then(() => {
          ws.send(JSON.stringify({ type: 'answer', answer: peerConnection.localDescription }));
        });
      break;

    case 'answer':
      peerConnection.setRemoteDescription(new RTCSessionDescription(message.answer));
      break;

    case 'candidate':
      peerConnection.addIceCandidate(new RTCIceCandidate(message.candidate));
      break;

    default:
      console.error('Unknown message type:', message.type);
  }
};

peerConnection.onicecandidate = (event) => {
  if (event.candidate) {
    ws.send(JSON.stringify({ type: 'candidate', candidate: event.candidate }));
  }
};

startButton.addEventListener('click', () => {
  peerConnection.createOffer()
    .then(offer => peerConnection.setLocalDescription(offer))
    .then(() => {
      ws.send(JSON.stringify({ type: 'offer', offer: peerConnection.localDescription }));
    });
  connectionStatus.textContent = "Connection started";
});

sendFileButton.addEventListener('click', () => {
  const file = fileInput.files[0];
  // Implement your file sending logic here
  // Possibly using a WebRTC data channel
});

// Disable the send file button until connection is established
sendFileButton.disabled = true;

peerConnection.ondatachannel = (event) => {
  // Implement logic to handle incoming data channels for file receiving
};

// You may also want to set up a data channel for file sending
