import Profile from "@/components/Profile.tsx";
import OngoingMedia from "@/components/OngoingMedia.tsx";

const Home = () => {
    return (
        <div className="grid grid-cols-2 container mx-auto p-6 items-start">
            <Profile />
            <OngoingMedia />
        </div>
    );
};
export default Home;
