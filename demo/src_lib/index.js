(async () => {
  // const a = await import('./async-a');

  setTimeout(async () => {
    const b = await import("./async-b");
    console.log(b);
  }, 5000);
})();
