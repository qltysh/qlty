import React, { useState } from "react";

const MyButton = () => {
  const [count, setCount] = useState(0);

  return (
    <div>
      <button onClick={() => setCount(count + 1)}>Click me!</button>
      <p>You've clicked the button {count} times.</p>
    </div>
  );
};

export default MyButton;
