import { For, Match, Show, Switch, createResource, createSignal } from 'solid-js'
import { type JSX } from 'solid-js/jsx-runtime'
import styles from './App.module.css'
import { clearAuthToken } from './auth'
import { HttpError, login, userInfo } from './fetch-utils'
import { type LoginResponse } from './models'

interface LoginProps {
    onSuccess: (response: LoginResponse) => void
}

function Login(props: LoginProps): JSX.Element {
    const [username, setUsername] = createSignal('')
    const [password, setPassword] = createSignal('')
    const [errorMessages, setErrorMessages] = createSignal<string[]>([])

    const submit = async (e: Event): Promise<void> => {
        try {
            e.preventDefault()
            const response = await login({
                username: username(),
                password: password(),
            })
            props.onSuccess(response)
        } catch (e) {
            setErrorMessages(HttpError.unwrap(e).messages)
        }
    }

    return <div class={styles.login}>
        <form onSubmit={(e) => void submit(e)}>
            <div class={styles.grid}>
                <label for='username'>Username:</label>
                <input
                    type='text'
                    placeholder='Username'
                    value={username()}
                    onChange={(e) => setUsername(e.target.value)}
                ></input>
                <label for='password'>Password:</label>
                <input
                    type='password'
                    placeholder='Password'
                    value={password()}
                    onChange={(e) => setPassword(e.target.value)}
                ></input>
            </div>
            <div class={styles.submitButton}>
                <button type="submit">Log In</button>
            </div>
        </form>
        <Show when={errorMessages().length > 0}>
            <div class={styles.errors}>
                <For each={errorMessages()}>{msg =>
                    <div>{msg}</div>
                }</For>
            </div>
        </Show>
    </div>
}

export function App(): JSX.Element {
    const [login, setLogin] = createSignal<LoginResponse | null>(null)

    const [userInfoResource] = createResource(true, async () => {
        setLogin(await userInfo())
    })

    const logout = (): void => {
        clearAuthToken()
        setLogin(null)
    }

    return <Switch>
        <Match when={userInfoResource.loading}>
            <div>Loading...</div>
        </Match>
        <Match when={login()}>
            <button onClick={logout}>Log Out</button>
            <div>Logged in: {login()?.username}</div>
        </Match>
        <Match when={!login()}>
            <Login onSuccess={setLogin}></Login>
        </Match>
    </Switch>
}
