import { createSignal } from "solid-js";

export default function Login() {
  const [username, setUsername] = createSignal<string>("");
  const [password, setPassword] = createSignal<string>("");

  const onClick = () => {
    const main = async () => {
      const res = await fetch("http://localhost:3000/login", {
        credentials: "include",
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          username: username(),
          password: password(),
        }),
      });
      if (res.status === 200) {
        window.location.href = decodeURIComponent(
          new URLSearchParams(window.location.search).get("to") || "/"
        );
      }
    };
    main();
  };

  return (
    <main class="text-center mx-auto p-4">
      <form class="flex flex-col mx-auto space-y-3 max-w-60">
        <input
          placeholder="Username"
          class="bg-crust text-subtext0 rounded-lg p-3"
          onChange={(e) => setUsername(e.currentTarget.value)}
        ></input>
        <input
          placeholder="Password"
          class="bg-crust text-subtext0 rounded-lg p-3"
          type="password"
          onChange={(e) => setPassword(e.currentTarget.value)}
        ></input>
        <button
          onClick={(e) => {
            e.preventDefault();
            onClick();
          }}
          class="text-text border-2 border-green hover:bg-mantle transition-all rounded-lg p-3"
        >
          Login
        </button>
      </form>
    </main>
  );
}
