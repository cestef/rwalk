import React, { useState, useEffect } from "react";

const TailwindSizeIndicator = () => {
	const [size, setSize] = useState("");

	useEffect(() => {
		const updateSize = () => {
			if (window.innerWidth >= 1280) {
				setSize("2xl");
			} else if (window.innerWidth >= 1024) {
				setSize("xl");
			} else if (window.innerWidth >= 768) {
				setSize("lg");
			} else if (window.innerWidth >= 640) {
				setSize("md");
			} else {
				setSize("sm");
			}
		};

		updateSize();
		window.addEventListener("resize", updateSize);

		return () => window.removeEventListener("resize", updateSize);
	}, []);

	return (
		<div className="fixed top-2 right-2 p-2 bg-gray-800 text-white rounded-lg shadow-md z-30">
			<span className="font-bold">{size}</span>
		</div>
	);
};

export default TailwindSizeIndicator;
