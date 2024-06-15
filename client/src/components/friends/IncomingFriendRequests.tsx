import { TOAST_ERROR_OPTIONS, TOAST_SUCCESS_OPTIONS } from "@/lib";
import useIncomingFriendRequests from "@/lib/hooks/useIncomingFriendRequests";
import call from "@/lib/call";
import toast from "react-hot-toast";

export default function IncomingFriendRequests() {
  const { isLoading, incomingFriendRequests } = useIncomingFriendRequests();

  const onClick = (sender: string, accept: boolean) => {
    return async (e: React.MouseEvent<HTMLButtonElement, MouseEvent>) => {
      e.preventDefault();
      const end = accept ? "accept" : "deny";
      (async () => {
        const res = await call(`/@me/friends/${sender}/${end}`, "POST");
        if (res.status === 200) {
          toast.success(`Friend request ${end}ed!`, TOAST_SUCCESS_OPTIONS);
        } else {
          toast.error(`Failed to ${end} friend request.`, TOAST_ERROR_OPTIONS);
        }
      })();
    };
  };

  if (isLoading) return <></>;

  return (incomingFriendRequests?.length || 0) > 0 ? (
    incomingFriendRequests?.map((request) => (
      <li
        className="flex flex-row space-x-3 justify-center"
        key={`${request.sender}-incoming`}
      >
        <p className="text-text">{request.sender}</p>
        <button
          onClick={onClick(request.sender, true)}
          className="text-mauve hover:text-pink transition-all"
        >
          {"["}Accept{"]"}
        </button>
        <button
          onClick={onClick(request.sender, false)}
          className="text-mauve hover:text-pink transition-all"
        >
          {"["}Deny{"]"}
        </button>
      </li>
    ))
  ) : (
    <></>
  );
}
