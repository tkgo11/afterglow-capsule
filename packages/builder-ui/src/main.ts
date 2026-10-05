import { mount } from "svelte";
import Scaffold from "./Scaffold.svelte";

const target = document.getElementById("app");
if (!target) throw new Error("Builder mount element is missing");
mount(Scaffold, { target });
