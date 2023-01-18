function getServerAddress() {
  server_ip = document.getElementById("server_id").value;
  var server_adress = 'http://'.concat(server_ip,':5000');
  return server_adress;
}

function outputToConsole(text) {
  var para = document.createElement("p");
  var node = document.createTextNode(text);
  para.appendChild(node);
    document.getElementById("console").appendChild(para);
    para.scrollIntoView();
}

class DoorapiREST {
  constructor(){
    document.getElementById("server_id").setAttribute('value','127.0.0.1');
  }
  get_status(){
    var xhttp = new XMLHttpRequest();
    xhttp.onreadystatechange = function() {
      if (this.readyState == 4 && this.status == 200) {
        alert(this.responseText);
      }
    };
    var ping_endpoint = "".concat(getServerAddress(),'/v1.0/ping');
    xhttp.open("GET", ping_endpoint, true);
    xhttp.setRequestHeader("Content-type", "application/json");
    xhttp.send(midi_msg); 
  }
}

let dooropen_api = new DoorapiREST();