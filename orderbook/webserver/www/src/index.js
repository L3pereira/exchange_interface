// import _ from 'lodash';
// function component() {
//     const element = document.createElement('h2');
  
//     // Lodash, now imported by this script
//     element.innerHTML = _.join(['Hello', 'lodash', 'webpackak'], ' ');
  
//     return element;
//   }
  
//   document.body.appendChild(component());

  function WebSocketTest() {
            
    if ("WebSocket" in window) {
      console.log("WebSocket is supported by your Browser!");
       
       // Let us open a web socket
       var ws = new WebSocket("ws://127.0.0.1:8080/stream");
  
       ws.onopen = function() {
          
          // Web Socket is connected, send data using send()
          ws.send("Message to send");
          console.log("Message is sent...");
       };
  
       ws.onmessage = function (evt) { 
          var received_msg = evt.data;
          console.log("Message is received..."+ evt);
       };
  
       ws.onclose = function() { 
          
          // websocket is closed.
          console.log("Connection is closed..."); 
       };
    } else {
      
       // The browser doesn't support WebSocket
       console.log("WebSocket NOT supported by your Browser!");
    }
  }
  function heartbeat() {
   if (!socket) return;
   if (socket.readyState !== 1) return;
   socket.send("heartbeat");
   setTimeout(heartbeat, 500);
  }
  WebSocketTest();