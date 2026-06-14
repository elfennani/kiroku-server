import createFetchClient from "openapi-fetch";
import createClient from "openapi-react-query";
import type { paths } from "./schema";

const fetchClient = createFetchClient<paths>({
  baseUrl: "/api",
});
const $api = createClient(fetchClient);

export default $api;
