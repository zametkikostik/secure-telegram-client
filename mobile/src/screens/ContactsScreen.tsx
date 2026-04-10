/**
 * Contacts Screen - заглушка
 */

import React from 'react';
import {View, Text, StyleSheet} from 'react-native';
import {Colors} from '../../utils/constants';

const ContactsScreen = () => {
  return (
    <View style={styles.container}>
      <Text style={styles.text}>Contacts Screen</Text>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: Colors.background,
    justifyContent: 'center',
    alignItems: 'center',
  },
  text: {
    color: Colors.text.primary,
    fontSize: 18,
  },
});

export default ContactsScreen;
