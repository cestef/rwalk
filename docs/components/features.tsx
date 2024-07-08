import { useId } from "react";
import {
	FastForward,
	Code,
	Blocks,
	Filter,
	RotateCcw,
	Folders,
	Link,
	FileText,
} from "lucide-react";
import { useConfig } from "nextra-theme-docs";

export function Feature({ text, icon }) {
	return (
		<div className="inline-flex items-center">
			{icon}
			<h4 className="ml-3 font-bold text-[0.9rem] md:text-[1.1rem]  whitespace-nowrap">
				{text}
			</h4>
		</div>
	);
}

/** @type {{ key: string; icon: React.FC }[]} */
const FEATURES_LIST = [
	{
		key: "fast",
		icon: <FastForward className="lg:w-6 w-4 sm:w-5 md:stroke-[2.5]" />,
	},
	{
		key: "scripting",
		icon: <Code className="lg:w-6 w-4 sm:w-5 md:stroke-[2.5]" />,
	},
	{
		key: "interactive",
		icon: <Blocks className="lg:w-6 w-4 sm:w-5 md:stroke-[2.5]" />,
	},
	{
		key: "filters",
		icon: <Filter className="lg:w-6 w-4 sm:w-5 md:stroke-[2.5]" />,
	},
	{
		key: "resume",
		icon: <RotateCcw className="lg:w-6 w-4 sm:w-5 md:stroke-[2.5]" />,
	},
	{
		key: "recursive",
		icon: <Folders className="lg:w-6 w-4 sm:w-5 md:stroke-[2.5]" />,
	},
	{
		key: "spider",
		icon: <Link className="lg:w-6 w-4 sm:w-5 md:stroke-[2.5]" />,
	},
	{
		key: "wordlists",
		icon: <FileText className="lg:w-6 w-4 sm:w-5 md:stroke-[2.5]" />,
	},
];

export default function Features() {
	const keyId = useId();
	const config = useConfig();

	const primaryColor = (lightness: string) =>
		`hsl(${config.primaryHue}deg, ${config.primarySaturation}%, ${lightness})`;

	const title = "Features";
	const features = {
		fast: "Blazingly fast",
		scripting: (
			<>
				Scripting in{" "}
				<a href="https://rhai.rs/" target="_blank" rel="noreferrer">
					<code
						className="font-mono font-bold underline"
						style={{
							color: primaryColor("50%"),
							// backgroundColor: primaryColor("12%"),
						}}
					>
						rhai
					</code>
				</a>
			</>
		),
		interactive: "Interactive mode",
		filters: "Advanced filters",
		resume: "Resume execution",
		recursive: "Recursive mode",
		spider: "Spider mode",
		wordlists: "Granular wordlists",
	};

	return (
		<div className="mx-auto max-w-full w-[900px] text-center px-4 mb-10">
			<p className="text-lg mb-4 text-gray-600 md:text-2xl">{title}</p>
			<div className="grid gap-y-4 gap-x-8 md:grid-cols-4 grid-cols-2 mb-8">
				{FEATURES_LIST.map(({ key, icon }) => (
					<Feature text={features[key]} icon={icon} key={keyId + key} />
				))}
			</div>
		</div>
	);
}
