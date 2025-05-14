namespace Lib

open System
open Microsoft.Xna.Framework
open Microsoft.Xna.Framework.Graphics
open Microsoft.Xna.Framework.Input

type DemoState(game: Game, graphics: GraphicsDeviceManager) =
    let spriteBatch = new SpriteBatch(graphics.GraphicsDevice)

    let ball = game.Content.Load<Texture2D> "ball"
    do printfn "TODO ball bounds = %A" ball.Bounds

    member this.Draw(gameTime: GameTime) : unit =
        graphics.GraphicsDevice.Clear Color.CornflowerBlue

        let width = graphics.GraphicsDevice.Adapter.CurrentDisplayMode.Width
        let height = graphics.GraphicsDevice.Adapter.CurrentDisplayMode.Height

        let ortho =
            Matrix.CreateOrthographicOffCenter(0.0f, 1024.0f, 0.0f, 768.0f, 0.0f, 10.0f)

        // spriteBatch.Begin(?transformMatrix = Some ortho)
        spriteBatch.Begin()
        spriteBatch.Draw(ball, new Vector2(0.0f, 0.0f), Color.White)
        spriteBatch.End()

type Demo() as this =
    inherit Game()

    let graphics = new GraphicsDeviceManager(this)
    let mutable state = None

    let mutable ball: Option<Texture2D> = None

    do this.Content.RootDirectory <- "Content"

    do graphics.PreferredBackBufferWidth <- 1024
    do graphics.PreferredBackBufferHeight <- 768
    do graphics.GraphicsProfile <- GraphicsProfile.HiDef
    do graphics.IsFullScreen <- false

    override this.Initialize() : unit = base.Initialize()

    override this.LoadContent() : unit =
        state <- Some(new DemoState(this, graphics))
        base.LoadContent()

    override this.Update(gameTime: GameTime) : unit =
        let keyboardState = Keyboard.GetState()

        if keyboardState.IsKeyDown Keys.Escape then
            this.Exit()

    override this.Draw(gameTime: GameTime) : unit =
        match state with
        | None -> ()
        | Some state -> state.Draw gameTime
