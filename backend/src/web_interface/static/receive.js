const configuration = { iceServers: [{ urls: 'stun:stun.l.google.com:19302' }] };
const peerConnection = new RTCPeerConnection(configuration);
let dataChannel;

const ws = new WebSocket('ws://127.0.0.1:9000');

const chunkSize = 16 * 1024;
const receivedChunks = [];

const sendWebSocketMessage = (type, payload) => ws.send(JSON.stringify({ type, ...payload }));

const handlers = {
  offer: async ({ offer }) => {
    console.log('Received offer, creating answer...');
    await peerConnection.setRemoteDescription(new RTCSessionDescription(offer));
    const answer = await peerConnection.createAnswer();
    await peerConnection.setLocalDescription(answer);
    sendWebSocketMessage('answer', { answer: peerConnection.localDescription });
  },
  answer: ({ answer }) => {
    if (peerConnection.signalingState !== 'have-local-offer') {
      console.error('Cannot set remote answer in current state:', peerConnection.signalingState);
      return;
    }
    peerConnection.setRemoteDescription(new RTCSessionDescription(answer));
  },
  candidate: ({ candidate }) => peerConnection.addIceCandidate(new RTCIceCandidate(candidate))
};

peerConnection.ondatachannel = event => {
    dataChannel = event.channel;
    setupDataChannel();
};

ws.addEventListener('message', event => {
  const message = JSON.parse(event.data);
  const handler = handlers[message.type];
  if (handler) {
    try {
      handler(message);
    } catch (error) {
      console.error('Error handling message:', message.type, error);
    }
  } else {
    console.error('Unknown message type:', message.type);
  }
});

peerConnection.addEventListener('icecandidate', event => {
  if (event.candidate) {
    sendWebSocketMessage('candidate', { candidate: event.candidate });
  }
});

function setupDataChannel() {
    dataChannel.addEventListener('open', () => {
    console.log('Data channel is open');
    });

    let hasProcessedMetadata = false;

    dataChannel.addEventListener('message', event => {
        if (!hasProcessedMetadata) {
            const [encodedFileName, fileSize] = event.data.split('|');
            const fileName = decodeURIComponent(encodedFileName);
            console.log(`Expecting file: ${fileName}, size: ${fileSize} bytes`);
            // Storing meta-data for later use
            receivedChunks.fileSize = Number(fileSize);
            receivedChunks.fileName = fileName;
            // Marking metadata as processed
            hasProcessedMetadata = true;
            return;
        }

        receivedChunks.push(event.data);
        const receivedSize = receivedChunks.reduce((total, chunk) => total + chunk.byteLength, 0);
        console.log(`Progress: ${(receivedSize / receivedChunks.fileSize * 100).toFixed(2)}%`);

        if (receivedSize >= receivedChunks.fileSize) {
            // All chunks have been received
            const receivedFile = new Blob(receivedChunks);
            const downloadLink = document.createElement('a');
            downloadLink.href = URL.createObjectURL(receivedFile);
            downloadLink.download = receivedChunks.fileName;
            document.body.appendChild(downloadLink);
            downloadLink.click();
            document.body.removeChild(downloadLink);
            console.log('File received successfully');
        }
    });

    dataChannel.addEventListener('error', error => {
    console.error('Data channel error:', error);
    });

    dataChannel.addEventListener('close', () => {
    console.log('Data channel state on close:', dataChannel.readyState);
    console.log('Peer connection state on close:', peerConnection.iceConnectionState);
    });
}