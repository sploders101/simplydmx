import { createApp } from "vue";
import "@/assets/theme.scss";
import "@/assets/style.scss";
import App from "./App.vue";
import "normalize.css";

import { registerGlobals } from "@/globalComponents";
import router from "./router";
import { createPinia } from "pinia";

let app = createApp(App);
app.use(router);
app.use(createPinia());
registerGlobals(app);
app.mount("#app");

const appHeight = () => {
	const doc = document.documentElement;
	doc.style.setProperty("--app-height", `${window.innerHeight}px`);
};
window.addEventListener("resize", appHeight);
appHeight();
