const ws = new WebSocket(`ws://${window.location.host}/ws`);
ws.onmessage = (event) => {
  console.log("Received:", event.data);
};
