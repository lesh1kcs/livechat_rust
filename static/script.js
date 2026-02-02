let socket;
let username;
let sessionId;

// Source - https://stackoverflow.com/a/24103596
// Posted by Mandeep Janjua, modified by community. See post 'Timeline' for change history
// Retrieved 2026-02-02, License - CC BY-SA 4.0

function setCookie(name, value, days) {
    var expires = "";
    if (days) {
        var date = new Date();
        date.setTime(date.getTime() + (days * 24 * 60 * 60 * 1000));
        expires = "; expires=" + date.toUTCString();
    }
    document.cookie = name + "=" + (value || "") + expires + "; path=/";
}

function getCookie(name) {
    var nameEQ = name + "=";
    var ca = document.cookie.split(';');
    for (var i = 0; i < ca.length; i++) {
        var c = ca[i];
        while (c.charAt(0) == ' ') c = c.substring(1, c.length);
        if (c.indexOf(nameEQ) == 0) return c.substring(nameEQ.length, c.length);
    }
    return null;
}

function eraseCookie(name) {
    document.cookie = name + '=; Path=/; Expires=Thu, 01 Jan 1970 00:00:01 GMT;';
}

document.addEventListener("DOMContentLoaded", async () => {
    sessionId = getCookie('session_id');

    if (!sessionId) {
        window.location.href = '/auth';
        return;
    }

    try {
        const response = await fetch('/api/user', {
            method: 'GET',
            credentials: 'include'
        });

        if (response.ok) {
            const data = await response.json();
            username = data.username;
            document.getElementById('user-info').textContent = `Logged in as: ${username}`;
            connectWebSocket();
        } else {
            eraseCookie('session_id');
            window.location.href = '/auth';
            return;
        }
    } catch (error) {
        console.error('Failed to fetch user:', error);
        window.location.href = '/auth';
        return;
    }

    const input = document.getElementById('message-input');
    input.addEventListener('keypress', (e) => {
        if (e.key === 'Enter') {
            sendMessage();
        }
    });
});

function connectWebSocket() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    socket = new WebSocket(`${protocol}//${window.location.host}/send`);

    socket.onopen = () => {
        console.log("WebSocket connection established");
        addSystemMessage('Connected to chat');
    };

    socket.onmessage = (event) => {
        console.log("Received:", event.data);
        addMessage(event.data);
    };

    socket.onclose = () => {
        console.log("WebSocket connection closed");
        addSystemMessage('Disconnected from chat');
    };

    socket.onerror = (error) => {
        console.error("WebSocket error:", error);
    };
}

function sendMessage() {
    const input = document.getElementById("message-input");

    if (!input || !input.value.trim()) return;

    if (!socket || socket.readyState !== WebSocket.OPEN) {
        console.error("Not connected!");
        addSystemMessage('Not connected');
        return;
    }

    const message = `${username}: ${input.value.trim()}`;
    socket.send(message);
    input.value = "";

    console.log(`Sent: ${message}`);
}

function addMessage(message) {
    const messagesDiv = document.getElementById('messages');
    const messageEl = document.createElement('div');
    messageEl.className = 'message';
    messageEl.textContent = message;
    messagesDiv.appendChild(messageEl);
    messagesDiv.scrollTop = messagesDiv.scrollHeight;
}

function addSystemMessage(message) {
    const messagesDiv = document.getElementById('messages');
    const messageEl = document.createElement('div');
    messageEl.className = 'message system';
    messageEl.textContent = `[System] ${message}`;
    messagesDiv.appendChild(messageEl);
    messagesDiv.scrollTop = messagesDiv.scrollHeight;
}

function logout() {
    if (socket) {
        socket.close();
    }
    eraseCookie('session_id');
    window.location.href = '/auth';
}