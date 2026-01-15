let socket;
let username;

document.getElementById("join-button").addEventListener("click", () => {
    const input = document.getElementById("username-input)")
    if (input.value.trim() !== ""){
        username = input.value.trim();
        document.getElementById("authentication-box").style.display = "none";
        document.getElementById("chat-messages").style.display = "block";
        connectWebSocket();
    }
});

username = prompt("Please enter your username:");
if (username && username.trim() !== "") {
    username = username.trim();
    document.getElementById("authentication-box").style.display = "none";
    document.getElementById("chat-messages").style.display = "block";
    connectWebSocket();
} else {
    alert("Username is required!");
    location.reload();
}

function connectWebSocket(){
    socket = new WebSocket("ws://localhost:3001/send");

    socket.onopen = () => {
        console.log("WebSocket connection established");
    };

    socket.onmessage = (event) => {
        const chatMessages = document.getElementById("chat-messages");
        const msgDiv = document.createElement("div");
        msgDiv.textContent = event.data;
        chatMessages.appendChild(msgDiv);
        chatMessages.scrollTop = chatMessages.scrollHeight;
    };
}

function sendMessage(){
    const input = document.getElementById("message-input");
    if (input.value.trim() && socket.readyState === WebSocket.OPEN) {
        const message = `${username}: ${input.value.trim()}`;
        socket.send(message);
        input.value = "";
    }
}

document.getElementById("send-button").addEventListener("click", sendMessage);
document.getElementById("message-input").addEventListener("keypress", (e) => {
    if (e.key === "Enter"){
        sendMessage();
    }
});
