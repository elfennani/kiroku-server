import useOngoingMediaQuery from "@/api/ongoing.query.ts";
import {LucideLoader2, LucideTriangleAlert} from "lucide-react";
import {Alert, AlertDescription, AlertTitle} from "@/components/ui/alert.tsx";
import {Link} from "react-router";

const OngoingMedia = () => {
    const {data, isPending, isError, error} = useOngoingMediaQuery();

    if (isPending) {
        return (
            <div className="w-full flex items-center justify-center h-64">
                <LucideLoader2 className="animate-spin"/>
            </div>
        )
    }

    if (isError) {
        return <Alert>
            <LucideTriangleAlert/>
            <AlertTitle>Failed to fetch ongoing media!</AlertTitle>
            <AlertDescription>{error.message}</AlertDescription>
        </Alert>
    }


    return (
        <div className="flex flex-col gap-4">
            {data.map(medium => (
                <Link
                    to={`/media/${medium.id}`} key={medium.id}
                    className="bg-secondary p-2 rounded-lg hover:bg-secondary/75 flex gap-2 items-start"
                >
                    <img
                        className="w-24 aspect-[0.69] object-cover rounded"
                        src={medium.cover?.thumbnail}
                        alt={medium.title}
                    />
                    <div>
                        <h2>{medium.title}</h2>
                        <p>{medium.status.progress} / {medium.status.total}</p>
                    </div>
                </Link>
            ))}
        </div>
    );
};
export default OngoingMedia;
