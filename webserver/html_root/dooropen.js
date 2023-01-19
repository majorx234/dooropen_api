function getServerAddress() {
  server_ip = document.getElementById("server_id").value;
  var server_adress = 'http://'.concat(server_ip,':8080');
  return server_adress;
}

function outputToConsole(text) {
  var para = document.createElement("p");
  var node = document.createTextNode(text);
  para.appendChild(node);
    document.getElementById("console").appendChild(para);
    para.scrollIntoView();
}

function serveGetRequest(endpoint) {
      var xhttp = new XMLHttpRequest();
    xhttp.onreadystatechange = function() {
      if (this.readyState == 4 && this.status == 200) {
        outputToConsole(this.responseText);
      }
    };
    var ping_endpoint = "".concat(getServerAddress(),endpoint);
    xhttp.open("GET", ping_endpoint, true);
    xhttp.setRequestHeader("Content-type", "application/json");
    xhttp.send(null);
}

class DoorapiREST {
  constructor(){
    document.getElementById("server_id").setAttribute('value','127.0.0.1');
  }
  ping(){
    outputToConsole('ping');
    serveGetRequest('/v1.0/ping');
  }
  get_door_status(){
    outputToConsole('get door_status:');
    serveGetRequest('/v1.0/door_status');
  }
}
outputToConsole("init");
let dooropen_api = new DoorapiREST();
document.getElementById('server_ping').onclick = function(){
  dooropen_api.ping();
}
document.getElementById('server_door_status').onclick = function() {
  dooropen_api.get_door_status();
};
outputToConsole("running...");
