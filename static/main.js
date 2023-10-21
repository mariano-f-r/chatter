const socket = new WebSocket("ws://"+window.location.host+"/ws");

socket.onmessage = function (event)  {
  let message = JSON.parse(event.data)
  console.log(message);
  let keys = Object.keys(message);
  if (keys[0] === "SystemMessage") {
    render_message(message.SystemMessage);
  } else if (keys[0]=== "ChatMessage") {
    render_message(message.ChatMessage.username+" at "+message.ChatMessage.time+": "+message.ChatMessage.content);
  } else if (keys[0] === "UserCountChange") {
    update_user_count(message.UserCountChange)
  }
}

function render_message(message) {
  const new_message = document.createElement("div");
  new_message.setAttribute("class", "box");
  // evil hack
  new_message.prepend(""+message);
  const msglog = document.getElementById("messages");
  msglog.appendChild(new_message);
  msglog.scrollTop = msglog.scrollHeight;
}

function update_user_count(count) {
  const counter = document.getElementById("usercount");
  counter.textContent="Users Online: "+count;
}

const message_field = document.getElementById("message");
message_field.addEventListener("keypress", function (event) {
  if (event.key === "Enter") {
    send(getName(), getMessage());
  }
});

/**
 * Checks that content of name and message input are between 0 and 32 or 256 respectively
 * @param   {String}  name  Content from name input box
 * @param   {String}  msg   Content from message input box
 * @return  {boolean}       True if the name and message meet conditions, false otherwise
 */
function validateMessage(name, msg) {
  if (name.length != 0 && name.length < 32 && msg.length != 0 && msg.length < 256) {
    return true;
  }
    return false;
  }
    
/**
 * Checks if name and message are valid
 * If valid visually and functionally enable send message button
 * If not valid disable button
 */
function updateButton() {
  const valid = validateMessage(getName(), getMessage());
  const send_button = document.getElementById("send_button");
  if (valid) {
    send_button.removeAttribute("disabled");
  } else {
    send_button.setAttribute("disabled", "");
  }
}

// Clears content of message input box
function clear() { 
  const message = document.getElementById("message");
  message.value = "";
}

// Returns content of name input box as String, whitespace on outside removed
function getName() {
  const name = document.getElementById("name");
  // console.log(name.value.trim());
  return name.value.trim();
}

// Returns content of message input box as String, whitespace on outside removed
function getMessage() {
  const msg = document.getElementById("message");
  // console.log(msg.value.trim());
  return msg.value.trim();
}

/**
 * Checks if name and message are valid, if true then send message, clear message input, and update send button
 * @param   {String}  name  Content from name input box
 * @param   {String}  msg   Content from message input box
 */
function send(name, msg) {
  if (validateMessage(name, msg) === true) {
    const date = new Date();
    // Stupid evil hack language
    const final_message = {ChatMessage: { username: name, time: date.toLocaleTimeString(), content: msg}};
    socket.send(JSON.stringify(final_message));
    clear();
    updateButton();
  }
}

