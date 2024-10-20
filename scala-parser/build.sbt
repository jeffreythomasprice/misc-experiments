val scala3Version = "3.5.1"

lazy val root = project
  .in(file("."))
  .settings(
    name := "scala-experiment",
    version := "0.1.0-SNAPSHOT",
    scalaVersion := scala3Version,
    libraryDependencies += "org.scalatest" %% "scalatest" % "3.2.18" % Test
  )
