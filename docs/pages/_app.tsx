import TailwindSizeIndicator from "../components/helper";
import "../styles.css";

const isDev = process.env.NODE_ENV === "development";

// This default export is required in a new `pages/_app.js` file.
export default function MyApp({ Component, pageProps }) {
	return (
		<>
			{isDev && <TailwindSizeIndicator />}
			<Component {...pageProps} />
		</>
	);
}
