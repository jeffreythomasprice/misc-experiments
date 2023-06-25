package main

import (
	"os"
	"path/filepath"
	"time"

	"go.uber.org/zap"
	"go.uber.org/zap/zapcore"
)

func createLogger() (*zap.Logger, func(), error) {
	// TODO rotate daily?
	const logPath = "logs/experiment.log"
	const timestampKey = "timestamp"

	// TODO name encoder?

	bigTimestampEncoderConfig := zap.NewProductionEncoderConfig()
	bigTimestampEncoderConfig.TimeKey = timestampKey
	bigTimestampEncoderConfig.EncodeTime = func(t time.Time, pae zapcore.PrimitiveArrayEncoder) {
		pae.AppendString(t.Format("2006-01-02T15:04:05.000Z0700"))
	}
	bigTimestampEncoderConfig.EncodeLevel = zapcore.CapitalLevelEncoder

	littleTimestampEncoderConfig := zap.NewProductionEncoderConfig()
	littleTimestampEncoderConfig.TimeKey = timestampKey
	littleTimestampEncoderConfig.EncodeTime = func(t time.Time, pae zapcore.PrimitiveArrayEncoder) {
		pae.AppendString(t.Format("2006-01-02T15:04:05Z"))
	}
	littleTimestampEncoderConfig.ConsoleSeparator = " "
	littleTimestampEncoderConfig.EncodeLevel = zapcore.CapitalColorLevelEncoder

	consoleEncoder := zapcore.NewConsoleEncoder(littleTimestampEncoderConfig)
	jsonEncoder := zapcore.NewJSONEncoder(bigTimestampEncoderConfig)

	errors := zap.LevelEnablerFunc(func(lvl zapcore.Level) bool {
		return lvl >= zapcore.ErrorLevel
	})
	notErrors := zap.LevelEnablerFunc(func(lvl zapcore.Level) bool {
		return lvl < zapcore.ErrorLevel
	})
	everything := zap.LevelEnablerFunc(func(lvl zapcore.Level) bool {
		return true
	})

	shutdownFuncs := []func(){}

	sinks := []zapcore.Core{
		zapcore.NewCore(consoleEncoder, zapcore.Lock(os.Stdout), notErrors),
		zapcore.NewCore(consoleEncoder, zapcore.Lock(os.Stderr), errors),
	}
	if !isDebug() {
		if err := os.MkdirAll(filepath.Dir(logPath), os.ModePerm); err != nil {
			return nil, func() {}, err
		}

		writeSyncer, close, err := zap.Open(logPath)
		if err != nil {
			return nil, func() {}, err
		}
		sinks = append(sinks, zapcore.NewCore(jsonEncoder, writeSyncer, everything))
		shutdownFuncs = append(shutdownFuncs, close)
	}

	result := zap.New(zapcore.NewTee(sinks...))
	return result,
		func() {
			result.Sync()
			for _, f := range shutdownFuncs {
				f()
			}
		},
		nil
}
