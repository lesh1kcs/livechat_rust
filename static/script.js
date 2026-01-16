let socket;
let username;

window.onload = function() {
    connectWebSocket();
};

document.addEventListener("DOMContentLoaded", () => {
    const joinButton = document.getElementById("join-button");
    const usernameinput = document.getElementById("username-input");

    if (joinButton){
        joinButton.addEventListener("click", handleJoin);
    }

    if (usernameinput){
        usernameinput.addEventListener("keypress", (e) => {
            if (e.key === "Enter"){
                handleJoin();
            }
        });
    }

    MessageListener();
});

function handleJoin(){
    const input = document.getElementById("username-input");
    if (input && input.value.trim() !== ""){
        username = input.value.trim();

        const authBox = document.getElementById("authentication-box");
        if (authBox){
            authBox.style.display = "none";
        }

        const chatBox = document.querySelector(".chat-box");
        if (chatBox){
            chatBox.style.display = "block";
        }
    }
}

function connectWebSocket(){
    console.log(`Connecting as: ${username}`);
    socket = new WebSocket("ws://localhost:3001/send");

    socket.onopen = () => {
        console.log("WebSocket connection established");
    };

    socket.onmessage = (event) => {
        displayMessage(event.data);
    };

    socket.onclose = () => {
        console.log("WebSocket connection closed");
    };

    socket.onerror = (error) => {
        console.error("WebSocket error:", error);
    };
}

function displayMessage(message){
    const chatMessages = document.getElementById("chat-messages");
    if(!chatMessages) return;

    const Divmsg = document.createElement("div");
    Divmsg.className = 'message';
    Divmsg.textContent = message;

    chatMessages.appendChild(Divmsg);
    chatMessages.scrollTop = chatMessages.scrollHeight;

    console.log(`Message: ${message}`);


}

function sendMessage(){
    const input = document.getElementById("message-input");

    if (!input || !input.value.trim()) return;
    if (!socket || socket.readyState !== WebSocket.OPEN) {
        console.error("❌ Not connected!");
        return;
    }

    const message = `${username}: ${input.value.trim()}`;
    socket.send(message);
    input.value = "";

    console.log(`Sent: ${message}`);

}

function MessageListener(){
    const sendButton = document.getElementById("send-button");
    if (sendButton){
        sendButton.addEventListener("click", sendMessage);
    }

    const messageInput = document.getElementById("message-input");
    if (messageInput){
        messageInput.addEventListener("keypress", (e) => {
            if (e.key === "Enter"){
                sendMessage();
            }
        });
    }
}
