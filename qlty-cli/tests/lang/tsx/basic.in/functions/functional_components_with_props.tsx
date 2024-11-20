import React from "react";

interface Props {
  name: string;
}

const Greeting: React.FC<Props> = ({ name }) => {
  return <div>Hello, {name}!</div>;
};

function Welcome(props: Props) {
  return <h1>Welcome, {props.name}</h1>;
}
