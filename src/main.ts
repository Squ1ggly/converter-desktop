import { mount } from "svelte";
import App from "./App.svelte";
import "./app.css";

const appRoot = document.getElementById("app");
if (!appRoot) {
  throw new Error("index.html is missing the #app element");
}

const app = mount(App, { target: appRoot });

export default app;
