/* @refresh reload */
import { render } from "solid-js/web";
import { lazy } from "solid-js";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./styles/global.css";
import App from "./App";

const SettingsApp = lazy(() => import("./SettingsApp"));
const HistoryApp = lazy(() => import("./HistoryApp"));

const root = document.getElementById("root");
const label = getCurrentWindow().label;

if (label === "settings") {
  render(() => <SettingsApp />, root!);
} else if (label === "history") {
  render(() => <HistoryApp />, root!);
} else {
  render(() => <App />, root!);
}
