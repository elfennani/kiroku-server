import {QueryClientProvider as MainQueryClientProvider, QueryClient} from "@tanstack/react-query";

const queryClient = new QueryClient();

export const QueryClientProvider = ({children}: { children: React.ReactNode }) => {
    return <MainQueryClientProvider client={queryClient}>{children}</MainQueryClientProvider>
}

export default queryClient;