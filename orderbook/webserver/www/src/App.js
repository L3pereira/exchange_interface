
import './App.css';
import React, { useState, useEffect,  useRef} from "react";


// import configData from "../config.json";
let symbols = ["usgd", "ujpy", "uaud", "ukrw", "umnt", "uchf", 
                "uhkd", "ucny", "uthb", "unok", "uusd", "ugbp", 
                "ucad", "usdr", "ueur", "usek", "uinr", "udkk"]

let prev_rates = symbols.reduce((acc, symbol) => {
  acc[symbol] = 0.0;
  return acc;
}, {});



function App() {
  const book = Array(41).fill(0);
  const [previous_rates, setPrevious_rates] = useState(prev_rates);

  const ws = useRef(null);

  useEffect(() => {    
    if ("WebSocket" in window) {
      console.log("WebSocket is supported by your Browser!");
      let url =  "@@@IP@@@";
      console.log("Websocket URL" + url );
      ws.current = new WebSocket(url);
      ws.current.onopen = function() {
         console.log("Connection Open");
       };       
       ws.current.onclose = function() { 
        console.log("Connection is closed..."); 
      };
 
    }
    else {
    
      // The browser doesn't support WebSocket
      console.log("WebSocket NOT supported by your Browser!");
   }
  });

  useEffect(() => {  
    ws.current.onmessage = function (message) { 
      // console.log(message);
      // console.log(message.data);
      if (!message.data ) 
        return;
      else if (message.data === 'PING'){
        // console.log("PING Received");
        ws.current.send("PONG");
        // console.log("PONG SENT");
        return;
      }
        

 

      const summary = JSON.parse(message.data);
      if (!summary) return;
      let spread = summary.spread;
    
      summary['asks'].forEach((level, index) => {
        let row = document.getElementById(index);
        row.getElementsByTagName('td')[1].innerHTML = level.price;
        row.getElementsByTagName('td')[2].innerHTML = level.amount;
        row.getElementsByTagName('td')[3].innerHTML = level.exchange;
        row.className = 'asks';
      });

      let row = document.getElementById(20);
      row.getElementsByTagName('td')[0].innerHTML = "Spread";
      row.getElementsByTagName('td')[1].innerHTML = spread;
      row.className = 'spread';

      summary['bids'].reverse().forEach((level, index) => {
        index = index + 21;
        let row = document.getElementById(index);
        row.getElementsByTagName('td')[1].innerHTML = level.price;
        row.getElementsByTagName('td')[2].innerHTML = level.amount;
        row.getElementsByTagName('td')[3].innerHTML = level.exchange;
        row.className = 'bids';
      });
      // console.log(myObj);
      // symbols.forEach(function(symbol){
      //       let previous = parseFloat(previous_rates[symbol]);
      //       let current = parseFloat(myObj[symbol]);
      //       let row = document.getElementById(symbol);
      //       if (current !== 0.0) {
      //          if (current > previous && previous !== 0.0){
      //             row.getElementsByTagName('td')[1].innerHTML = current;
      //             row.className = 'increase';

      //          }
      //          else if (current < previous && previous !== 0.0){
      //             row.getElementsByTagName('td')[1].innerHTML = current;
      //             row.className = 'decrease';
      //          }
      //          else {
      //             row.getElementsByTagName('td')[1].innerHTML = current;
      //             row.className = '';
      //          }
      //       }

      //       previous_rates[symbol] = current;
      //       setPrevious_rates(previous_rates);

      //     });
    };  
  });



 
  return (
      <div>
        <h1>Aggregated Orderbook</h1>
        <table  id="rates">
          <thead>
            <tr><th></th><th>Price</th><th>Volume</th><th>Exchange</th></tr>
          </thead>
          <tbody>
            { 
              book.map((row, index) => {
   
                  return <tr id={index} key={index}><td>{""}</td><td>{""}</td><td>{""}</td><td>{""}</td></tr>
              })

              
            }
          </tbody>
        </table>
      </div>
    )

}

export default App;
