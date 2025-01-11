const socket = new WebSocket('ws://localhost:8080/ws');

const chatBox = document.getElementById('chat-box');
const messageInput = document.getElementById('message-input');
const sendButton = document.getElementById('send-button');

function addMessage(text, isSender = false) {
	const messageElement = document.createElement('div');
	messageElement.className = `message ${isSender ? 'sender' : ''}`;
	messageElement.textContent = text;
	chatBox.appendChild(messageElement);
	chatBox.scrollTop = chatBox.scrollHeight;
}

sendButton.addEventListener('click', () => {
	const message = messageInput.value.trim();
	if (message) {
		addMessage(`You: ${message}`, true);
		socket.send(message); // Send message to WebSocket
		messageInput.value = '';
	}
});

socket.addEventListener('message', (event) => {
	addMessage(`Someone: ${event.data}`);
});

socket.addEventListener('open', () => {
	console.log('Connected to chat server');
	addMessage('Connected to the chat server.');
});

socket.addEventListener('close', () => {
	addMessage('Disconnected from the chat server.');
});
