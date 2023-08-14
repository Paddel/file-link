const configuration = { iceServers: [{ urls: 'stun:stun.l.google.com:19302' }] };
const peerConnection = new RTCPeerConnection(configuration);
const dataChannel = peerConnection.createDataChannel('fileTransfer');

const [fileInput, startButton, sendFileButton, connectionStatus] =
  ['fileInput', 'startButton', 'sendFile', 'connectionStatus'].map(id => document.getElementById(id));
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

sendFileButton.disabled = true;

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

dataChannel.addEventListener('open', () => {
  console.log('Data channel is open');
  sendFileButton.disabled = false;
});

dataChannel.addEventListener('message', event => {
  console.log('Received Message:', event.data.byteLength, 'bytes');
  alert('received')
  receivedChunks.push(event.data);
  
  if (receivedChunks.length === 1) {
    const fileMeta = new TextDecoder().decode(receivedChunks[0]);
    const [fileName, fileSize] = fileMeta.split('|');
    console.log(`Expecting file: ${fileName}, size: ${fileSize} bytes`);
    // Storing meta-data for later use
    receivedChunks.fileSize = Number(fileSize);
    receivedChunks.fileName = fileName;
    // Removing the meta-data chunk
    receivedChunks.shift();
  }

  // Calculating total received size
  const receivedSize = receivedChunks.reduce((total, chunk) => total + chunk.byteLength, 0);

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


startButton.addEventListener('click', async () => {
  try {
    const offer = await peerConnection.createOffer();
    await peerConnection.setLocalDescription(offer);
    sendWebSocketMessage('offer', { offer: peerConnection.localDescription });
    connectionStatus.textContent = "Connection started";
  } catch (error) {
    console.error('Error starting connection:', error);
  }
});

sendFileButton.addEventListener('click', () => {
  console.log('Sending file...');
  const file = fileInput.files[0];
  const fileReader = new FileReader();
  let offset = 0;

  // Construct and send the metadata
  const encodedFileName = encodeURIComponent(file.name);
  const fileMeta = `${encodedFileName}|${file.size}`;
  dataChannel.send(fileMeta);
  console.log(`Sent metadata: ${fileMeta}`);

  const readSlice = o => {
    const slice = file.slice(offset, o + chunkSize);
    fileReader.readAsArrayBuffer(slice);
  };

  fileReader.onload = event => {
    dataChannel.send(event.target.result);
    offset += chunkSize;
    if (offset < file.size) {
      readSlice(offset);
    } else {
      console.log('File sent successfully');
    }
  };

  readSlice(0);
  console.log(`File size: ${file.size} bytes`);
});