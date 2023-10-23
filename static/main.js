const socket = new WebSocket("wss://"+window.location.host+"/ws");

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
  } else if (keys[0] === "TypingEvent") {
    if (message.TypingEvent.is_starting) {
      render_new_typer(message.TypingEvent.username);
    } else {
      destroy_typer(message.TypingEvent.username);
    }
  }
}

// Since many events are attached to the message element,
// better to just use an event listener
const message_field = document.getElementById("message");

message_field.addEventListener("keypress", function (event) {
  if (event.key === "Enter") {
    send(getName(), getMessage());
  }
});

// Will send ws messages to the server alerting it that this user is starting to type,
// will also update button
let typingTimeout;
let isTyping = false;
message_field.addEventListener("input", function () {
  if (getName()=="") {
    return;
  }
  start_typing();
  if (typingTimeout != undefined) {
    clearTimeout(typingTimeout);
  }
  typingTimeout = setTimeout(stop_typing, 1000);
  updateButton();
});

function render_message(message) {
  const new_message = document.createElement("div");
  new_message.setAttribute("class", "box");
  // evil hack
  new_message.prepend(""+message);
  const msglog = document.getElementById("messages");
  msglog.appendChild(new_message);
  msglog.scrollTop = msglog.scrollHeight;
}

function render_new_typer(username) {
  const new_typer = document.createElement("div");
  new_typer.setAttribute("class", "box");
  new_typer.innerHTML = "<strong>"+username+"</strong> is typing";
  const typer_list = document.getElementById("typing");
  typer_list.appendChild(new_typer);
}

function destroy_typer(username) {
  const typer_list = document.getElementById("typing");
  for (let i = 0; i<typer_list.children.length; i++) {
    if (typer_list.children[i].firstChild.innerText == username) {
      typer_list.children[i].remove();
      return;
    }
  }
}

function update_user_count(count) {
  const counter = document.getElementById("usercount");
  counter.textContent="Users Online: "+count;
}

function start_typing () {
  if (!isTyping) {
    send_typing_event(true);
    isTyping=true;
  }
}

function stop_typing () {
  isTyping = false;
  send_typing_event(false);
}
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
function send_message(name, msg) {
  if (validateMessage(name, msg) === true) {
    const date = new Date();
    // Stupid evil hack language
    const final_message = {ChatMessage: { username: name, time: date.toLocaleTimeString(), content: msg}};
    socket.send(JSON.stringify(final_message));
    clear();
    updateButton();
  }
}

function send_typing_event(starting) {
  const message = {TypingEvent: {username: getName(), is_starting: starting,}};
  socket.send(JSON.stringify(message));
}
