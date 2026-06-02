import { useState } from "react";
import { createApiClient, type LoginResponse } from "@fullstack-rust-react-starter/api-client";

const API_BASE = import.meta.env.VITE_API_BASE_URL ?? "http://localhost:8080";

export default function App() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [session, setSession] = useState<LoginResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function handleRegister() {
    setLoading(true);
    setError(null);
    try {
      const api = createApiClient(API_BASE);
      const data = await api.register({
        email,
        password,
        display_name: email.split("@")[0] || "用户",
      });
      setSession(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : "注册失败");
    } finally {
      setLoading(false);
    }
  }

  async function handleLogin() {
    setLoading(true);
    setError(null);
    try {
      const api = createApiClient(API_BASE);
      const data = await api.login({ email, password });
      setSession(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : "登录失败");
    } finally {
      setLoading(false);
    }
  }

  return (
    <main className="page">
      <h1>Fullstack Rust React Starter</h1>
      <p className="muted">用户注册 / 登录（默认角色 user）</p>

      {!session ? (
        <section className="card">
          <label>
            邮箱
            <input value={email} onChange={(e) => setEmail(e.target.value)} placeholder="you@example.com" />
          </label>
          <label>
            密码（至少 8 位）
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="********"
            />
          </label>
          {error && <p className="error">{error}</p>}
          <div className="actions">
            <button type="button" onClick={handleLogin} disabled={loading}>
              登录
            </button>
            <button type="button" className="secondary" onClick={handleRegister} disabled={loading}>
              注册
            </button>
          </div>
        </section>
      ) : (
        <section className="card">
          <p>
            已登录：<strong>{session.display_name}</strong>（{session.email}）
          </p>
          <p className="muted">角色：{session.roles.join(", ") || "无"}</p>
          <button type="button" onClick={() => setSession(null)}>
            退出
          </button>
        </section>
      )}
    </main>
  );
}
