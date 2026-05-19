import { useParams } from "react-router";
import { LucideLoader2, LucideTriangleAlert } from "lucide-react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert.tsx";
import useProcessedMediaQuery from "@/api/processed.query.ts";

const MediaRoute = () => {
  const { id } = useParams();
  const { data, isPending, isError, error } = useProcessedMediaQuery(
    Number(id),
  );

  if (isPending) {
    return (
      <Layout>
        <div className="w-full flex items-center justify-center h-64">
          <LucideLoader2 className="animate-spin" />
        </div>
      </Layout>
    );
  }

  if (isError) {
    return (
      <Layout>
        <Alert>
          <LucideTriangleAlert />
          <AlertTitle>Failed to fetch processed media!</AlertTitle>
          <AlertDescription>{error.message}</AlertDescription>
        </Alert>
      </Layout>
    );
  }

  return (
    <Layout>
      <div className="flex flex-col gap-2 w-full">
        {data.map((media) => (
          <div key={media.id} className="flex flex-col gap-1">
            <h2 className="font-medium">Episode {media.episode}</h2>
            <p className="text-secondary-foreground text-sm">
              {Math.round(media.duration / (1000 * 1000 * 60))} minutes
            </p>
          </div>
        ))}
      </div>
    </Layout>
  );
};

const Layout = ({ children }: { children: React.ReactNode }) => {
  return <div className="max-w-md mx-auto p-6">{children}</div>;
};

export default MediaRoute;
