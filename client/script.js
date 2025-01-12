document.getElementById('join-button').onclick = function () {
	const username = document.getElementById('username-input').value.trim();
	if (username) {
		// Hide the username input page
		document.getElementById('username-container').style.display = 'none';

		// Show the chat page
		document.getElementById('chat-container').style.display = 'block';

		handle_websocket(username);
	}
};

function handle_websocket(username) {
	const socket = new WebSocket('ws://localhost:8080/ws');

	const messageInput = document.getElementById('message-input');
	const sendButton = document.getElementById('send-button');

	sendButton.onclick = function () {
		const message = messageInput.value.trim();
		if (message) {
			socket.send(JSON.stringify({ message }));
			messageInput.value = '';
		}
	};

	socket.onopen = function () {
		socket.send(JSON.stringify({ username }));
	};

	socket.onclose = function () {
		addMessage('System', 'Disconnected from the chat server.');
	};

	socket.onmessage = function (event) {
		let data = JSON.parse(event.data);
		if (data.username === username) {
			data.username = 'You';
		}
		addMessage(data.username, data.message);
	};
}

function addMessage(username, message) {
	const messageElement = document.createElement('div');
	messageElement.className = `message ${username}`;
	messageElement.textContent = `${username}: ${message}`;

	const chatBox = document.getElementById('chat-box');
	chatBox.appendChild(messageElement);
	chatBox.scrollTop = chatBox.scrollHeight;
}
