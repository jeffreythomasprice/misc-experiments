namespace Experiment.Server

open Microsoft.Extensions.Logging
open Microsoft.IdentityModel.Tokens
open System
open System.IdentityModel.Tokens.Jwt
open System.Security.Claims
open System.Text

type JWTService(log: ILogger<JWTService>) =
    // TODO use certs?
    let signingKey =
        SymmetricSecurityKey(Encoding.UTF8.GetBytes("TODO signing key some more bits to get the key size up enough"))

    member this.tokenValidationParameters =
        TokenValidationParameters(
            ValidateIssuer = false,
            ValidateAudience = false,
            ValidateLifetime = true,
            ValidateIssuerSigningKey = true,
            IssuerSigningKey = signingKey
        )

    member this.createToken(username: string) =
        let expirationTime = DateTime.Now.AddHours 1

        let result =
            JwtSecurityToken(
                null,
                null,
                [ Claim("username", username) ],
                DateTime.Now,
                expirationTime,
                this.signingCredentials
            )

        let result = (JwtSecurityTokenHandler().WriteToken(result), expirationTime)
        log.LogTrace("issued {token}", result)
        result

    member this.validateToken(token: string) =
        try
            Some(JwtSecurityTokenHandler().ValidateToken(token, this.tokenValidationParameters))
        with e ->
            log.LogDebug("error validating token {token}: {e}", token, e.Message)
            None

    member private this.signingCredentials =
        SigningCredentials(signingKey, SecurityAlgorithms.HmacSha256)
