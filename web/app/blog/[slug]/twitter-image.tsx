// Twitter gets the same per-post branded card as OpenGraph — including the build-time
// prerender (generateStaticParams), so it's cached rather than rendered on demand.
export { default, alt, size, contentType, generateStaticParams } from "./opengraph-image";
