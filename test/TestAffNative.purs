-- Differential tests for the native Node.FS.Aff blocking path against the
-- previous Node.FS.Async `makeAff` implementations. Both paths run on the same
-- host, so the oracle also proves the JavaScript host still uses the fallback.
module TestAffNative where

import Prelude

import Data.Array as Array
import Data.Either (Either(..), isLeft)
import Effect (Effect)
import Effect.Aff (Aff, Error, attempt, launchAff_, makeAff, nonCanceler)
import Effect.Aff as Aff
import Effect.Class (liftEffect)
import Effect.Console (log)
import Node.Encoding (Encoding(..))
import Node.FS.Aff as FSA
import Node.FS.Async as A
import Node.FS.Stats (Stats)
import Node.FS.Stats as Stats
import Node.FS.Sync as S
import Test.Assert (assert', assertEqual')

-- The previous Node.FS.Aff implementations, kept as the differential oracle.
oldToAff1 :: forall a x. (x -> A.Callback a -> Effect Unit) -> x -> Aff a
oldToAff1 f x = makeAff \k -> f x k $> nonCanceler

oldToAff2 :: forall a x y. (x -> y -> A.Callback a -> Effect Unit) -> x -> y -> Aff a
oldToAff2 f x y = makeAff \k -> f x y k $> nonCanceler

oldToAff3 :: forall a x y z. (x -> y -> z -> A.Callback a -> Effect Unit) -> x -> y -> z -> Aff a
oldToAff3 f x y z = makeAff \k -> f x y z k $> nonCanceler

messageOf :: forall a. Either Error a -> String
messageOf = case _ of
  Left error -> Aff.message error
  Right _ -> "<success>"

statsFields :: Stats -> { isFile :: Boolean, size :: Number }
statsFields stats = { isFile: Stats.isFile stats, size: Stats.size stats }

sameFailure :: forall a. String -> Aff a -> Aff a -> Aff Unit
sameFailure label fresh oracle = do
  freshResult <- attempt fresh
  oracleResult <- attempt oracle
  liftEffect do
    assert' (label <> ": native path must fail") (isLeft freshResult)
    assert' (label <> ": oracle path must fail") (isLeft oracleResult)
    assertEqual' label { actual: messageOf freshResult, expected: messageOf oracleResult }

sameStats :: String -> Aff Stats -> Aff Stats -> Aff Unit
sameStats label fresh oracle = do
  freshStats <- fresh
  oracleStats <- oracle
  liftEffect $ assertEqual' label { actual: statsFields freshStats, expected: statsFields oracleStats }

main :: Effect Unit
main = launchAff_ do
  base <- liftEffect $ S.mkdtemp "test/node-fs-tests-native-aff-"
  let
    plain = base <> "/plain.txt"
    body = "hello native aff"
    unicodeName = "naïve-文件-🙂.txt"
    unicode = base <> "/" <> unicodeName
    unicodeBody = "unicode body"
    sub = base <> "/sub"
    missing = base <> "/missing.txt"

  -- writeTextFile (native) then read back with the old Async path, and
  -- readTextFile (native) compared with the old Async path.
  _ <- FSA.writeTextFile UTF8 plain body
  oldText <- oldToAff2 A.readTextFile UTF8 plain
  newText <- FSA.readTextFile UTF8 plain
  liftEffect do
    assertEqual' "old Async reads the native write" { actual: oldText, expected: body }
    assertEqual' "native read matches the body" { actual: newText, expected: body }

  -- Deferral and replay: building the Aff must not touch the file, and every
  -- run must replay the write.
  let deferred = base <> "/deferred.txt"
  let deferredWrite = FSA.writeTextFile UTF8 deferred "deferred"
  beforeExists <- liftEffect $ S.exists deferred
  liftEffect $ assert' "a deferred write must not run at construction" (not beforeExists)
  _ <- deferredWrite
  firstRun <- FSA.readTextFile UTF8 deferred
  _ <- FSA.writeTextFile UTF8 deferred "changed"
  _ <- deferredWrite
  secondRun <- FSA.readTextFile UTF8 deferred
  liftEffect do
    assertEqual' "first run writes" { actual: firstRun, expected: "deferred" }
    assertEqual' "a replay re-executes the write" { actual: secondRun, expected: "deferred" }

  -- mkdir / stat: compare the relevant Stats fields, never the timestamps.
  _ <- FSA.mkdir sub
  sameStats "stat directory" (FSA.stat sub) (oldToAff1 A.stat sub)
  sameStats "stat file" (FSA.stat plain) (oldToAff1 A.stat plain)

  -- Unicode names survive both paths and enumerate identically.
  _ <- FSA.writeTextFile UTF8 unicode unicodeBody
  oldUnicode <- oldToAff2 A.readTextFile UTF8 unicode
  newUnicode <- FSA.readTextFile UTF8 unicode
  freshNames <- FSA.readdir base
  oracleNames <- oldToAff1 A.readdir base
  liftEffect do
    assertEqual' "old Async reads the Unicode file" { actual: oldUnicode, expected: unicodeBody }
    assertEqual' "native reads the Unicode file" { actual: newUnicode, expected: unicodeBody }
    assertEqual' "readdir matches the Async path" { actual: Array.sort freshNames, expected: Array.sort oracleNames }
    assert' "readdir lists the Unicode name" (Array.elem unicodeName freshNames)

  -- Error paths: ENOENT, EEXIST, EISDIR/ENOTDIR share the exact message.
  sameFailure "stat ENOENT" (FSA.stat missing) (oldToAff1 A.stat missing)
  sameFailure "readTextFile ENOENT" (FSA.readTextFile UTF8 missing) (oldToAff2 A.readTextFile UTF8 missing)
  sameFailure "mkdir EEXIST" (FSA.mkdir sub) (oldToAff1 A.mkdir sub)
  sameFailure "writeTextFile EISDIR" (FSA.writeTextFile UTF8 sub "x") (oldToAff3 A.writeTextFile UTF8 sub "x")
  sameFailure "readdir ENOTDIR" (FSA.readdir plain) (oldToAff1 A.readdir plain)

  liftEffect $ S.rm' base { force: false, maxRetries: 100, recursive: true, retryDelay: 1000 }
  liftEffect $ log "FS Aff native: all good"
