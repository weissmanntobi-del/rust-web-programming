document.querySelector('#load').addEventListener('click', async () => {
  const response = await fetch('/api/tasks');
  const data = await response.json();
  document.querySelector('#output').textContent = JSON.stringify(data, null, 2);
});
