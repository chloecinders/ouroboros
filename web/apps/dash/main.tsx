import { render } from "solid-js/web";
import { App } from "./App.tsx";

const root = document.getElementById("app");

if (!root) throw new Error("no #app found");

render(App, root);
