// Share modality picker (PRD §6.5.1). Each action first creates a tracked share
// link, then performs the action so analytics attribute the visit correctly.
//
// QR and the OS share sheet (which itself covers AirDrop / iMessage / Email /
// Copy) are implemented here with no native dependency. NFC tap and direct
// Add-to-Wallet require native modules (react-native-nfc-manager, a PassKit
// bridge) and a dev build — they attach in the hardware-integration milestone.

import { useState } from "react";
import { Linking, Modal, Pressable, Share, StyleSheet, Text, View } from "react-native";
import * as Haptics from "expo-haptics";
import { createShareLink } from "../../api/share";
import { googleWalletSaveUrl } from "../../api/cards";
import { QRShareOverlay } from "./QRShareOverlay";

interface Props {
  cardId: string;
  handle: string;
  visible: boolean;
  onClose: () => void;
}

export function ShareSheet({ cardId, handle, visible, onClose }: Props) {
  const [qrUrl, setQrUrl] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const showQr = async () => {
    setBusy(true);
    try {
      const link = await createShareLink(cardId, "qr");
      Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Light);
      setQrUrl(link.url);
    } finally {
      setBusy(false);
    }
  };

  const shareLink = async () => {
    setBusy(true);
    try {
      const link = await createShareLink(cardId, "link");
      await Share.share({ message: link.url, url: link.url });
    } finally {
      setBusy(false);
    }
  };

  const addToGoogleWallet = async () => {
    setBusy(true);
    try {
      const url = await googleWalletSaveUrl(cardId);
      await Linking.openURL(url);
    } finally {
      setBusy(false);
    }
  };

  return (
    <Modal visible={visible} animationType="slide" transparent onRequestClose={onClose}>
      {qrUrl ? (
        <QRShareOverlay url={qrUrl} handle={handle} onClose={() => setQrUrl(null)} />
      ) : (
        <Pressable style={styles.backdrop} onPress={onClose}>
          <Pressable style={styles.sheet} onPress={(e) => e.stopPropagation()}>
            <View style={styles.grabber} />
            <Text style={styles.title}>Share your card</Text>
            <Action label="Show QR code" onPress={showQr} disabled={busy} />
            <Action label="Share link…" onPress={shareLink} disabled={busy} />
            <Action label="Add to Google Wallet" onPress={addToGoogleWallet} disabled={busy} />
            <Pressable style={styles.cancel} onPress={onClose}>
              <Text style={styles.cancelText}>Cancel</Text>
            </Pressable>
          </Pressable>
        </Pressable>
      )}
    </Modal>
  );
}

function Action({ label, onPress, disabled }: { label: string; onPress: () => void; disabled?: boolean }) {
  return (
    <Pressable style={[styles.action, disabled && styles.disabled]} onPress={onPress} disabled={disabled}>
      <Text style={styles.actionText}>{label}</Text>
    </Pressable>
  );
}

const styles = StyleSheet.create({
  backdrop: { flex: 1, backgroundColor: "rgba(0,0,0,0.5)", justifyContent: "flex-end" },
  sheet: {
    backgroundColor: "#15151a",
    borderTopLeftRadius: 24,
    borderTopRightRadius: 24,
    padding: 20,
    paddingBottom: 36,
    gap: 10,
  },
  grabber: {
    alignSelf: "center",
    width: 40,
    height: 4,
    borderRadius: 2,
    backgroundColor: "#3a3a40",
    marginBottom: 8,
  },
  title: { color: "#f5f5f7", fontSize: 20, fontWeight: "700", marginBottom: 8 },
  action: { backgroundColor: "#222228", borderRadius: 14, paddingVertical: 16, alignItems: "center" },
  actionText: { color: "#f5f5f7", fontSize: 16, fontWeight: "600" },
  disabled: { opacity: 0.4 },
  cancel: { paddingVertical: 16, alignItems: "center" },
  cancelText: { color: "#9a9aa0", fontSize: 16 },
});
