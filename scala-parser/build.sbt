ThisBuild / version := "0.1.0-SNAPSHOT"

ThisBuild / scalaVersion := "3.3.4"

lazy val root = (project in file("."))
	.settings(
		name := "scala-parser",
		libraryDependencies += "org.scalatest" %% "scalatest" % "3.2.18" % Test,
		libraryDependencies += "org.scalatestplus" %% "scalacheck-1-18" % "3.2.19.0" % "test"
	)
