import { useRouter } from "expo-router";
import { useState } from "react";
import {
  ActivityIndicator,
  Pressable,
  StyleSheet,
  Text,
  TextInput,
  View,
} from "react-native";
import { login, register } from "../../src/api/auth";
import { validateHandle } from "../../src/utils/handleValidation";

export default function LoginScreen() {
  const router = useRouter();
  const [mode, setMode] = useState<"login" | "register">("login");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [handle, setHandle] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const submit = async () => {
    setError(null);
    if (mode === "register") {
      const handleErr = validateHandle(handle);
      if (handleErr) {
        setError(`Handle invalid: ${handleErr.replace(/_/g, " ")}`);
        return;
      }
    }
    setBusy(true);
    try {
      if (mode === "register") {
        await register({ email, password, handle, displayName });
      } else {
        await login(email, password);
      }
      router.replace("/");
    } catch {
      setError(mode === "login" ? "Invalid email or password" : "Could not create account");
    } finally {
      setBusy(false);
    }
  };

  return (
    <View style={styles.root}>
      <Text style={styles.title}>CardClaws</Text>
      <Text style={styles.tagline}>The other side of you.</Text>

      {mode === "register" && (
        <>
          <Field placeholder="Display name" value={displayName} onChangeText={setDisplayName} />
          <Field
            placeholder="Handle (cardclaws.com/you)"
            value={handle}
            onChangeText={setHandle}
            autoCapitalize="none"
          />
        </>
      )}
      <Field
        placeholder="Email"
        value={email}
        onChangeText={setEmail}
        autoCapitalize="none"
        keyboardType="email-address"
      />
      <Field placeholder="Password" value={password} onChangeText={setPassword} secureTextEntry />

      {error && <Text style={styles.error}>{error}</Text>}

      <Pressable style={styles.primary} onPress={submit} disabled={busy}>
        {busy ? (
          <ActivityIndicator color="#fff" />
        ) : (
          <Text style={styles.primaryText}>{mode === "login" ? "Sign in" : "Create account"}</Text>
        )}
      </Pressable>

      <Pressable onPress={() => setMode(mode === "login" ? "register" : "login")}>
        <Text style={styles.switch}>
          {mode === "login" ? "Need an account? Register" : "Have an account? Sign in"}
        </Text>
      </Pressable>
    </View>
  );
}

function Field(props: React.ComponentProps<typeof TextInput>) {
  return (
    <TextInput
      {...props}
      style={styles.input}
      placeholderTextColor="#6b6b70"
    />
  );
}

const styles = StyleSheet.create({
  root: { flex: 1, justifyContent: "center", padding: 24, gap: 12, backgroundColor: "#0a0a0c" },
  title: { color: "#f5f5f7", fontSize: 40, fontWeight: "800", letterSpacing: -1 },
  tagline: { color: "#9a9aa0", marginBottom: 24 },
  input: {
    backgroundColor: "#1a1a1f",
    color: "#f5f5f7",
    borderRadius: 12,
    paddingHorizontal: 16,
    paddingVertical: 14,
    fontSize: 16,
  },
  error: { color: "#ff453a" },
  primary: {
    backgroundColor: "#ff3b30",
    borderRadius: 14,
    paddingVertical: 16,
    alignItems: "center",
    marginTop: 8,
  },
  primaryText: { color: "#fff", fontWeight: "700", fontSize: 17 },
  switch: { color: "#9a9aa0", textAlign: "center", marginTop: 16 },
});
